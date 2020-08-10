use appstream_rs::{AppId, Collection, Component};
use flatpak::prelude::*;
use flatpak::InstallationExt;
use flatpak::Remote;
use gio::prelude::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub struct AppStreamCache {
    collections: HashMap<Remote, Collection>,
}

impl AppStreamCache {
    pub fn new() -> Rc<Self> {
        let installation = flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();
        let remotes = installation.list_remotes(Some(&gio::Cancellable::new())).unwrap();
        let mut collections = HashMap::new();

        // Parse data
        for remote in remotes {
            let appstream_dir = remote.get_appstream_dir(Some("x86_64")).unwrap();

            let mut path = PathBuf::new();
            path.push(appstream_dir.get_path().unwrap().to_str().unwrap());
            path.push("appstream.xml");

            match Collection::from_path(path.clone()) {
                Ok(collection) => {
                    info!("Loaded appstream data: {:?}", &path);
                    collections.insert(remote.clone(), collection);
                }
                Err(_) => warn!("No appstream data: {:?}", &path),
            }
        }

        Rc::new(Self { collections })
    }

    pub fn get_components(&self, app_id: AppId) -> HashMap<flatpak::Remote, Component> {
        let mut components = HashMap::new();

        for (remote, collection) in &self.collections {
            let mut iter = collection.components.iter();
            iter.find(|component| component.id == app_id).map(|component| {
                components.insert(remote.clone(), component.clone());
            });
        }

        components
    }
}
