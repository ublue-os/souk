use appstream::enums::Bundle;
use appstream::Collection;
use flatpak::prelude::*;
use flatpak::InstallationExt;
use gio::prelude::*;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;

use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::database::queries;
use crate::database::{DbInfo, DbPackage};

lazy_static! {
    pub static ref DB_VERSION: String = "1.0".to_string();

    // Database lifetime in hours, before it gets rebuilt
    pub static ref DB_LIFETIME: i64 = 3;
}

pub fn init(flatpak_backend: Rc<FlatpakBackend>) {
    if needs_rebuilt() {
        debug!("Database needs rebuilt.");
        rebuild(flatpak_backend.clone());
    } else {
        debug!("Database it up-to-date, don't rebuild it.");
        //send!(DatabaseIsReady)
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
                dbi.clone()[0].clone()
            }
        }
        Err(_) => return true,
    };

    // Check database version
    if db_info.db_version != DB_VERSION.to_string() {
        debug!("Database version mismatch: {} != {}", db_info.db_version, DB_VERSION.to_string());
        return true;
    }

    // Check database lifetime
    let timestamp: i64 = db_info.db_timestamp.parse::<i64>().unwrap();
    let database_dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
    let now_dt: DateTime<Utc> = chrono::offset::Utc::now();
    if (now_dt - database_dt) > Duration::hours(*DB_LIFETIME) {
        debug!("Database lifetime exceeded");
        return true;
    }

    return false;
}

// TODO: Make this asynchronous, and report parsing progress to UI
pub fn rebuild(flatpak_backend: Rc<FlatpakBackend>) {
    debug!("Rebuild package database.");

    // Delete previous data
    queries::reset().unwrap();

    // Get all remotes (user/system)
    let mut system_remotes = flatpak_backend.system_installation.list_remotes(Some(&gio::Cancellable::new())).unwrap();
    let mut user_remotes = flatpak_backend.user_installation.list_remotes(Some(&gio::Cancellable::new())).unwrap();

    // TODO: Avoid parsing the same remote 2 times, when it
    // exists in `system` and `user` flatpak installation
    let mut remotes = Vec::new();
    remotes.append(&mut system_remotes);
    remotes.append(&mut user_remotes);

    debug!("Query {} remotes for packages...", remotes.len());

    for remote in remotes {
        let default_arch = "x86_64"; // TODO: use flatpak::get_default_arch();

        // Get path of appstream data
        let mut appstream_file = PathBuf::new();
        let appstream_dir = remote.get_appstream_dir(Some(default_arch)).unwrap();
        appstream_file.push(appstream_dir.get_path().unwrap().to_str().unwrap());
        appstream_file.push("appstream.xml");

        // Parse appstream xml to collection
        match Collection::from_path(appstream_file.clone()) {
            Ok(collection) => {
                debug!("Parsed appstream data: {:?}", &appstream_file);
                // Iterate appstream components, and look for components which we need
                // We're only interested in Flatpak stuff
                for component in collection.components {
                    let bundle = &component.bundles[0];
                    match bundle {
                        Bundle::Flatpak { runtime: _, sdk: _, reference: _ } => {
                            let package = Package::new(component.clone(), remote.clone().get_name().unwrap().to_string());
                            let db_package = DbPackage::from_package(&package);
                            match queries::insert_db_package(db_package) {
                                Ok(_) => (),
                                Err(err) => debug!("Unable to insert db package: {}", err.to_string()),
                            };
                        }
                        _ => debug!("Ignore non Flatpak component: {}", component.id.0),
                    }
                }
            }
            Err(err) => warn!("Unable to parse appstream for remote {:?}: {}", &remote.get_name().unwrap().to_string(), err.to_string()),
        }
    }

    // Set database info
    let mut db_info = DbInfo::default();
    db_info.db_version = DB_VERSION.to_string();
    db_info.db_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();
    queries::insert_db_info(db_info).unwrap();

    debug!("Finished rebuilding database.")
}
