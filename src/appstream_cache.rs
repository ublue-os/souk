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
    system_install: flatpak::Installation,
}

impl AppStreamCache {
    pub fn new() -> Rc<Self> {
        let system_install = flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();
        let remotes = system_install.list_remotes(Some(&gio::Cancellable::new())).unwrap();
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

        Rc::new(Self { collections, system_install })
    }

    /// checks if an app is installed (optionally a remote can specified)
    pub fn is_installed (&self, app_id: AppId, remote: Option<&flatpak::Remote>) -> bool {
        let mut result = false;

        let installed_refs = self.get_installed_refs();
        for package in installed_refs {
            result = package.get_name().unwrap().to_string() == app_id.0;

            remote.map(|remote|{
                result = remote.get_name().unwrap().to_string() == package.get_origin().unwrap().to_string();
            });
        }
        result
    }

    pub fn get_installed_refs(&self) -> Vec<flatpak::InstalledRef>{
        self.system_install.list_installed_refs(Some(&gio::Cancellable::new())).unwrap()
    }

    pub fn get_system_remotes(&self) -> Vec<flatpak::Remote>{
        self.system_install.list_remotes(Some(&gio::Cancellable::new())).unwrap()
    }

    /// Returns appstream components for a specific AppId
    pub fn get_components_for_app_id(&self, app_id: AppId) -> HashMap<flatpak::Remote, Component> {
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
