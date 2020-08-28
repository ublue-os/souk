use appstream_rs::{Bundle, Collection};
use flatpak::prelude::*;
use flatpak::{Installation, InstallationExt, RefKind};
use gio::prelude::*;
use bus::{Bus, BusReader};

use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::backend::Package;

pub enum PackageState{
    Installed,
    Installing,
    UpdateAvailable,
    Updating,
    Removing,
    NotInstalled,
    Unknown,
}

pub enum BackendMessage{
    Installed,
    Removed,
}

pub struct FlatpakBackend {
    system_installation: Installation,
    user_installation: Installation,

    message_bus: RefCell<Bus<BackendMessage>>,
    packages: RefCell<HashMap<String, Package>>,
}

impl FlatpakBackend {
    pub fn new() -> Rc<Self> {
        let system_installation = flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();

        let mut user_path = glib::get_home_dir().unwrap();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        let user_installation = flatpak::Installation::new_for_path(&gio::File::new_for_path(user_path), true, Some(&gio::Cancellable::new())).unwrap();

        let message_bus = RefCell::new(Bus::new(10));
        let packages = RefCell::new(HashMap::new());

        let backend = Rc::new(Self {
            system_installation,
            user_installation,
            message_bus,
            packages
        });

        backend.clone().reload_appstream_data();
        backend
    }

    /// Returns receiver which can be used to subscribe to backend messages.
    /// Receives message when something happens on Flatpak side (e.g. install/uninstall/update/...)
    pub fn get_message_receiver(self: Rc<Self>) -> BusReader<BackendMessage>{
        self.message_bus.borrow_mut().add_rx()
    }

    pub fn get_installed_packages(self: Rc<Self>) -> Vec<Package> {
        let mut installed_packages = Vec::new();

        let mut system_refs = self.system_installation.list_installed_refs(Some(&gio::Cancellable::new())).unwrap();
        let mut user_refs = self.user_installation.list_installed_refs(Some(&gio::Cancellable::new())).unwrap();

        let mut installed_refs = Vec::new();
        installed_refs.append(&mut system_refs);
        installed_refs.append(&mut user_refs);

        for ref_ in installed_refs{
            let kind = match ref_.get_kind(){
                RefKind::App => "app".to_string(),
                RefKind::Runtime => "runtime".to_string(),
                _ => "unknown".to_string(),
            };
            let name = ref_.get_name().unwrap().to_string();
            let arch = ref_.get_arch().unwrap().to_string();
            let branch = ref_.get_branch().unwrap().to_string();
            let origin = ref_.get_origin().unwrap().to_string();

            match self.clone().get_package(kind, name.clone(), arch, branch){
                Some(package) => installed_packages.insert(0, package.clone()),
                None => warn!("Unable to get package for flatpak ref {} ({})", name, origin),
            }
        }

        installed_packages
    }

    pub fn get_package(self: Rc<Self>, kind: String, name: String, arch: String, branch: String) -> Option<Package>{
        let id = format!("{}/{}/{}/{}", kind, name, arch, branch);
        match self.packages.borrow().get(&id){
            Some(package) => Some(package.to_owned()),
            None => None,
        }
    }

    pub fn is_package_installed(self: Rc<Self>, package: &Package) -> bool {
        let mut result = false;

        let installed_packages = self.clone().get_installed_packages();
        let mut iter = installed_packages.into_iter();
        iter.find(|p| package == p).map(|_| {
            result = true;
            return result;
        });

        result
    }

    pub fn install_package(self: Rc<Self>, _package: Package) {

    }

    fn reload_appstream_data(self: Rc<Self>) {
        let mut packages: HashMap<String, Package> = HashMap::new();

        // Get all remotes (user/system)
        let mut system_remotes = self.system_installation.list_remotes(Some(&gio::Cancellable::new())).unwrap();
        let mut user_remotes = self.user_installation.list_remotes(Some(&gio::Cancellable::new())).unwrap();

        let mut remotes = Vec::new();
        remotes.append(&mut system_remotes);
        remotes.append(&mut user_remotes);

        for remote in remotes{
            // Get path of appstream data
            let mut appstream_file = PathBuf::new();
            let appstream_dir = remote.get_appstream_dir(Some("x86_64")).unwrap();
            appstream_file.push(appstream_dir.get_path().unwrap().to_str().unwrap());
            appstream_file.push("appstream.xml");

            // Parse appstream xml to collections
            match Collection::from_path(appstream_file.clone()) {
                Ok(collection) => {
                    info!("Loaded appstream data: {:?}", &appstream_file);
                    // Iterate appstream components, and look for components which we need
                    for component in collection.components{
                        let bundle = &component.bundle[0];
                        match bundle {
                            Bundle::Flatpak{runtime: _, sdk: _, id} => {
                                let package = Package::new(component.clone(), remote.clone());
                                packages.insert(id.clone(), package);
                            },
                            _ => debug!("Ignore non Flatpak component: {}", component.id.0),
                        }
                    }
                }
                Err(_) => warn!("Unable to load appstream data: {:?}", &appstream_file),
            }
        }

        *self.packages.borrow_mut() = packages;
    }
}
