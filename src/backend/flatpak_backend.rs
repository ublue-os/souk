use broadcaster::BroadcastChannel;
use flatpak::prelude::*;
use flatpak::{Installation, InstallationExt};

use std::rc::Rc;
use std::sync::Arc;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{BackendMessage, Package, PackageAction, PackageTransaction};
use crate::database::package_database;
use crate::database::queries;

pub struct FlatpakBackend {
    pub system_installation: Installation,
    pub user_installation: Installation,

    transaction_backend: Box<dyn TransactionBackend>,
    broadcast: BroadcastChannel<BackendMessage>,
}

impl FlatpakBackend {
    pub fn new() -> Rc<Self> {
        let system_installation =
            flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();

        let mut user_path = glib::get_home_dir().unwrap();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        let user_installation = flatpak::Installation::new_for_path(
            &gio::File::new_for_path(user_path),
            true,
            Some(&gio::Cancellable::new()),
        )
        .unwrap();

        let broadcast = BroadcastChannel::new();

        let transaction_backend = if Self::is_sandboxed() {
            Box::new(SandboxBackend::new())
        } else {
            unimplemented!("Host backend not implemented yet");
        };

        let backend = Rc::new(Self {
            system_installation,
            user_installation,
            transaction_backend,
            broadcast,
        });

        package_database::init(backend.clone());

        backend
    }

    /// Returns receiver which can be used to subscribe to backend messages.
    /// Receives message when something happens on Flatpak side (e.g. install/uninstall/update/...)
    //pub fn get_message_receiver(self: Rc<Self>) -> BusReader<BackendMessage> {
    //self.message_bus.borrow_mut().add_rx()
    //}

    pub fn get_channel(self: Rc<Self>) -> BroadcastChannel<BackendMessage> {
        self.broadcast.clone()
    }

    pub fn get_installed_packages(self: Rc<Self>) -> Vec<Package> {
        let mut installed_packages = Vec::new();

        let mut system_refs = self
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();
        let mut user_refs = self
            .user_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        let mut installed_refs = Vec::new();
        installed_refs.append(&mut system_refs);
        installed_refs.append(&mut user_refs);

        for ref_ in installed_refs {
            let name = ref_.get_name().unwrap().to_string();
            let branch = ref_.get_branch().unwrap().to_string();
            let origin = ref_.get_origin().unwrap().to_string();

            if let Some(package) = queries::get_package(name, branch, origin).unwrap() {
                installed_packages.insert(0, package.clone())
            }
        }

        installed_packages
    }

    pub fn is_package_installed(self: Rc<Self>, package: &Package) -> bool {
        let mut result = false;

        let installed_packages = self.get_installed_packages();
        let mut iter = installed_packages.into_iter();
        iter.find(|p| package == p).map(|_| {
            result = true;
            result
        });

        result
    }

    pub fn install_package(self: Rc<Self>, package: Package) {
        let transaction = PackageTransaction::new(package, PackageAction::Install);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        self.transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn uninstall_package(self: Rc<Self>, package: Package) {
        let transaction = PackageTransaction::new(package, PackageAction::Uninstall);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        self.transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn launch_package(self: Rc<Self>, package: Package) {
        match std::process::Command::new("flatpak-spawn")
            .arg("--host")
            .arg("flatpak")
            .arg("run")
            .arg(package.get_ref_name())
            .spawn()
        {
            Ok(_) => (),
            Err(err) => warn!(
                "Unable to launch {}: {}",
                package.get_ref_name(),
                err.to_string()
            ),
        };
    }

    pub fn cancel_package_transaction(self: Rc<Self>, transaction: Arc<PackageTransaction>){
        self.transaction_backend.cancel_package_transaction(transaction);
    }

    fn send_message(self: Rc<Self>, message: BackendMessage) {
        let future = async move {
            self.broadcast.send(&message).await.unwrap();
        };
        spawn!(future);
    }

    fn is_sandboxed() -> bool {
        std::path::Path::new("/.flatpak-info").exists()
    }
}
