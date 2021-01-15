use appstream::enums::Bundle;
use appstream::{AppId, Collection, Component};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use flatpak::prelude::*;
use flatpak::InstallationExt;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;
use std::time::SystemTime;

use crate::backend::{SoukFlatpakBackend, SoukPackage};
use crate::db::queries;
use crate::db::DbInfo;

lazy_static! {
    pub static ref DB_VERSION: String = "1.1".to_string();

    // Database lifetime in hours, before it gets rebuilt
    pub static ref DB_LIFETIME: i64 = 3;
}

#[derive(Clone)]
enum DbMessage {
    PopulatingStarted,
    PopulatingEnded,
    Percentage(f64),
    Remote(String),
}

pub struct SoukDatabasePrivate {
    busy: Cell<bool>,
    percentage: Cell<f64>,
    remote: Rc<RefCell<String>>,

    sender: glib::Sender<DbMessage>,
    receiver: RefCell<Option<glib::Receiver<DbMessage>>>,
}

static PROPERTIES: [subclass::Property; 2] = [
    subclass::Property("percentage", |percentage| {
        glib::ParamSpec::double(
            percentage,
            "Percentage",
            "Percentage",
            0.0,
            1.0,
            0.0,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("remote", |remote| {
        glib::ParamSpec::string(remote, "Remote", "Remote", None, glib::ParamFlags::READABLE)
    }),
];

impl ObjectSubclass for SoukDatabasePrivate {
    const NAME: &'static str = "SoukDatabase";
    type Type = SoukDatabase;
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib::object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
        klass.add_signal(
            "populating-started",
            glib::SignalFlags::ACTION,
            &[],
            glib::Type::Unit,
        );
        klass.add_signal(
            "populating-ended",
            glib::SignalFlags::ACTION,
            &[],
            glib::Type::Unit,
        );
    }

    fn new() -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_LOW);

        SoukDatabasePrivate {
            busy: Cell::default(),
            percentage: Cell::default(),
            remote: Rc::default(),
            sender,
            receiver: RefCell::new(Some(receiver)),
        }
    }
}

impl ObjectImpl for SoukDatabasePrivate {
    fn get_property(&self, _obj: &SoukDatabase, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("percentage", ..) => self.percentage.get().to_value(),
            subclass::Property("remote", ..) => self.remote.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib::wrapper! {
    pub struct SoukDatabase(ObjectSubclass<SoukDatabasePrivate>);
}

#[allow(dead_code)]
impl SoukDatabase {
    pub fn new() -> Self {
        let database = glib::Object::new::<Self>(&[]).unwrap();

        let self_ = SoukDatabasePrivate::from_instance(&database);
        let receiver = self_.receiver.borrow_mut().take().unwrap();
        let db = database.clone();
        receiver.attach(None, move |msg| db.process_db_message(msg));

        database
    }

    pub fn init(&self) {
        if Self::needs_rebuilt() {
            debug!("Database needs rebuilt.");
            self.populate_database();
        } else {
            debug!("Database it up-to-date, don't repopulate it.");

            let self_ = SoukDatabasePrivate::from_instance(self);
            self_.busy.set(false);
            self.emit("populating-ended", &[]).unwrap();
        }
    }

    pub fn needs_rebuilt() -> bool {
        // Try to get db info
        let db_info = match queries::get_db_info() {
            Ok(dbi) => {
                if dbi.is_empty() {
                    debug!("Database is empty.");
                    return true;
                } else {
                    dbi[0].clone()
                }
            }
            Err(_) => return true,
        };

        // Check database version
        if db_info.db_version != DB_VERSION.to_string() {
            debug!(
                "Database version mismatch: {} != {}",
                db_info.db_version,
                DB_VERSION.to_string()
            );
            return true;
        }

        // Check database lifetime
        let timestamp: i64 = db_info.db_timestamp.parse::<i64>().unwrap();
        let database_dt =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
        let now_dt: DateTime<Utc> = chrono::offset::Utc::now();
        if (now_dt - database_dt) > Duration::hours(*DB_LIFETIME) {
            debug!("Database lifetime exceeded");
            return true;
        }

        false
    }

    pub fn populate_database(&self) {
        debug!("Populating / refreshing package database.");

        let self_ = SoukDatabasePrivate::from_instance(self);
        let sender = self_.sender.clone();

        if self_.busy.get() {
            info!("Database is already populating, skip.");
            return;
        }

        thread::spawn(move || {
            send!(sender, DbMessage::PopulatingStarted);

            let mut remotes = Vec::new();
            let flatpak_backend = SoukFlatpakBackend::new();

            // Reset data
            send!(sender, DbMessage::Remote("".to_string()));
            send!(sender, DbMessage::Percentage(0.0));

            // Delete previous data
            queries::reset().unwrap();

            // Package count to calculcate progress
            let mut pkg_count = 0;
            let mut pkg_pos = 0;

            // Get remotes
            let result = flatpak_backend
                .get_system_installation()
                .list_remotes(gio::NONE_CANCELLABLE)
                .unwrap();

            for remote in result {
                let name = remote.get_name().unwrap().to_string();
                let refs = flatpak_backend
                    .get_system_installation()
                    .list_remote_refs_sync(&name, gio::NONE_CANCELLABLE);

                match refs {
                    Ok(refs) => {
                        remotes.insert(0, remote);
                        pkg_count = pkg_count + refs.len();
                    }
                    Err(err) => warn!("Unable to load remote \"{}\": {}", name, err.to_string()),
                }
            }

            debug!(
                "Loading {} remotes with {} packages...",
                remotes.len(),
                pkg_count
            );

            for remote in remotes {
                let url = remote.get_url().unwrap().to_string();
                let remote_name = remote.get_name().unwrap().to_string();

                debug!("Load remote \"{}\" ({})", remote_name, url);
                send!(sender, DbMessage::Remote(remote_name.clone()));

                // Get all refs from remote
                let refs = flatpak_backend
                    .get_system_installation()
                    .list_remote_refs_sync(&remote_name, gio::NONE_CANCELLABLE)
                    .unwrap();

                // Get path of remote appstream data
                let mut appstream_file = PathBuf::new();
                let appstream_dir = remote
                    .get_appstream_dir(Some(std::env::consts::ARCH))
                    .unwrap();
                appstream_file.push(appstream_dir.get_path().unwrap().to_str().unwrap());
                appstream_file.push("appstream.xml");

                // Parse remote appstream data
                debug!("Parse appstream XML {:?}...", appstream_file);
                let appdata_collection = match Collection::from_path(appstream_file.clone()) {
                    Ok(collection) => {
                        debug!(
                            "Successfully parsed appstream XML for remote {}",
                            remote_name
                        );
                        Some(collection)
                    }
                    Err(err) => {
                        warn!(
                            "Unable to parse appstream XML for remote {:?}: {}",
                            err.to_string(),
                            remote_name
                        );
                        None
                    }
                };

                let mut db_packages = Vec::new();
                for remote_ref in refs {
                    let ref_name = remote_ref.format_ref().unwrap().to_string();
                    debug!("Found package {}", ref_name);

                    // We only care about our arch
                    if remote_ref.get_arch().unwrap().to_string() != std::env::consts::ARCH {
                        pkg_pos += 1;
                        continue;
                    }

                    // Try to retrieve appstream component
                    let component: Option<Component> = match &appdata_collection {
                        Some(collection) => {
                            let app_id = AppId(remote_ref.get_name().unwrap().to_string());
                            let components = collection.find_by_id(app_id);

                            components
                                .into_iter()
                                .find(|c| Self::get_ref_name(c) == ref_name)
                                .cloned()
                        }
                        None => {
                            warn!("Unable to find appstream data for {}", ref_name);
                            None
                        }
                    };

                    // Create new remote package and push it into the databse
                    let package = SoukPackage::from((
                        remote_ref,
                        serde_json::to_string(&component).unwrap().to_string(),
                    ));
                    db_packages.push(package.into());

                    pkg_pos += 1;

                    // Calculate percentage
                    let percentage = pkg_pos as f64 / pkg_count as f64;
                    send!(sender, DbMessage::Percentage(percentage));
                }

                match queries::insert_db_packages(db_packages) {
                    Ok(_) => (),
                    Err(err) => debug!("Unable to insert db packages: {}", err.to_string()),
                };
            }

            // Set database info
            let mut db_info = DbInfo::default();
            db_info.db_version = DB_VERSION.to_string();
            db_info.db_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();
            queries::insert_db_info(db_info).unwrap();

            debug!("Finished populating database.");
            send!(sender, DbMessage::PopulatingEnded);
        });
    }

    fn process_db_message(&self, msg: DbMessage) -> glib::Continue {
        let self_ = SoukDatabasePrivate::from_instance(self);

        match msg {
            DbMessage::PopulatingStarted => {
                self_.busy.set(true);
                self.emit("populating-started", &[]).unwrap();
            }
            DbMessage::PopulatingEnded => {
                self_.busy.set(false);
                self.emit("populating-ended", &[]).unwrap();
            }
            DbMessage::Percentage(p) => {
                self_.percentage.set(p);
                self.notify("percentage");
            }
            DbMessage::Remote(remote) => {
                *self_.remote.borrow_mut() = remote;
                self.notify("remote");
            }
        }

        glib::Continue(true)
    }

    fn get_ref_name(component: &Component) -> String {
        for bundle in &component.bundles {
            match bundle {
                Bundle::Flatpak {
                    runtime: _,
                    sdk: _,
                    reference,
                } => return reference.clone(),
                _ => (),
            }
        }
        String::new()
    }

    pub fn get_percentage(&self) -> f64 {
        self.get_property("percentage")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_remote(&self) -> String {
        self.get_property("remote").unwrap().get().unwrap().unwrap()
    }
}
