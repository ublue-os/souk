use appstream::enums::Bundle;
use appstream::{AppId, Collection, Component};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use flatpak::prelude::*;
use flatpak::{InstallationExt, Remote};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use once_cell::unsync::OnceCell;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;

use crate::backend::{SoukFlatpakBackend, SoukPackage};
use crate::db::queries;
use crate::db::DbInfo;

lazy_static! {
    pub static ref DB_VERSION: String = "1.1".to_string();

    // Database lifetime in hours, before it gets rebuilt
    pub static ref DB_LIFETIME: i64 = 3;
}

pub struct SoukDatabasePrivate {
    busy: Rc<RefCell<bool>>,
    flatpak_backend: OnceCell<SoukFlatpakBackend>,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property("busy", |busy| {
    glib::ParamSpec::boolean(busy, "Busy", "Busy", false, glib::ParamFlags::READABLE)
})];

impl ObjectSubclass for SoukDatabasePrivate {
    const NAME: &'static str = "SoukDatabase";
    type Type = SoukDatabase;
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        SoukDatabasePrivate {
            busy: Rc::default(),
            flatpak_backend: OnceCell::default(),
        }
    }
}

impl ObjectImpl for SoukDatabasePrivate {
    fn get_property(&self, _obj: &SoukDatabase, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("busy", ..) => self.busy.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukDatabase(ObjectSubclass<SoukDatabasePrivate>);
}

#[allow(dead_code)]
impl SoukDatabase {
    pub fn new(flatpak_backend: SoukFlatpakBackend) -> Self {
        let database = glib::Object::new(SoukDatabase::static_type(), &[])
            .unwrap()
            .downcast::<SoukDatabase>()
            .unwrap();

        let self_ = SoukDatabasePrivate::from_instance(&database);
        self_.flatpak_backend.set(flatpak_backend).unwrap();

        database
    }

    pub fn init(&self) {
        if Self::needs_rebuilt() {
            debug!("Database needs rebuilt.");
            self.rebuild();
        } else {
            debug!("Database it up-to-date, don't rebuild it.");
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

    // TODO: Make this asynchronous, and report parsing progress to UI
    pub fn rebuild(&self) {
        let self_ = SoukDatabasePrivate::from_instance(self);
        debug!("Rebuild package database.");

        // Delete previous data
        queries::reset().unwrap();

        let mut remotes: HashMap<String, Remote> = HashMap::new();

        // Get system remotes
        let system_remotes = self_
            .flatpak_backend
            .get()
            .unwrap()
            .get_system_installation()
            .list_remotes(Some(&gio::Cancellable::new()))
            .unwrap();
        for remote in system_remotes {
            remotes.insert(remote.get_name().unwrap().to_string(), remote);
        }

        // TODO: Add support for user remotes
        // Get user remotes
        //let user_remotes = self.flatpak_backend
        //    .user_installation
        //    .list_remotes(Some(&gio::Cancellable::new()))
        //    .unwrap();
        //for remote in user_remotes {
        //    remotes.insert(remote.get_name().unwrap().to_string(), remote);
        //}

        for remote in remotes {
            let url = remote.1.get_url().unwrap().to_string();
            let remote_name = remote.1.get_name().unwrap().to_string();

            if url.starts_with("oci+") {
                // TODO: Add support for OCI remotes
                // https://gitlab.gnome.org/haecker-felix/souk/-/issues/11#note_925791
                warn!(
                    "Unable to load remote \"{}\" ({}): OCI remotes aren't supported yet",
                    remote_name, url
                );
                continue;
            }

            debug!("Load remote \"{}\" ({})", remote_name, url);

            // Get all refs from remote
            let refs = self_
                .flatpak_backend
                .get()
                .unwrap()
                .get_system_installation()
                .list_remote_refs_sync(&remote_name, Some(&gio::Cancellable::new()));

            if let Err(err) = refs {
                warn!(
                    "Unable to retrieve refs from remote \"{}\": {}",
                    remote.1.get_name().unwrap().to_string(),
                    err.to_string()
                );
                continue;
            }
            let refs = refs.unwrap();

            // Get path of appstream data
            let mut appstream_file = PathBuf::new();
            let appstream_dir = remote
                .1
                .get_appstream_dir(Some(std::env::consts::ARCH))
                .unwrap();
            appstream_file.push(appstream_dir.get_path().unwrap().to_str().unwrap());
            appstream_file.push("appstream.xml");

            // Parse appstream data
            let appdata_collection = Collection::from_path(appstream_file.clone()).ok();
            if appdata_collection.is_none() {
                warn!(
                    "Unable to parse appstream for remote {:?}",
                    &remote.1.get_name().unwrap().to_string()
                );
            }

            let mut db_packages = Vec::new();
            let count = refs.len();
            let mut pos = 0.0;

            for remote_ref in refs {
                let ref_name = remote_ref.format_ref().unwrap().to_string();
                debug!(
                    "[{}%] Load package {}",
                    (pos / count as f32) * 100.0,
                    ref_name
                );

                // We only care about our arch
                if remote_ref.get_arch().unwrap().to_string() != std::env::consts::ARCH {
                    pos = pos + 1.0;
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
                    None => None,
                };

                // Create new remote package and push it into the databse
                let package = SoukPackage::from((
                    remote_ref,
                    serde_json::to_string(&component).unwrap().to_string(),
                ));
                db_packages.push(package.into());

                pos = pos + 1.0;
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

        debug!("Finished rebuilding database.")
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
}
