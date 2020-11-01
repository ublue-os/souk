use broadcaster::BroadcastChannel;
use flatpak::{Installation, InstallationExt, InstalledRef, RefExt};
use gio::prelude::*;

use std::rc::Rc;
use std::sync::Arc;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{
    BackendMessage, InstalledPackage, Package, PackageAction, PackageKind, PackageTransaction,
};
use crate::database::{package_database, DisplayLevel};

pub struct FlatpakBackend {
    pub system_installation: Installation,
    system_monitor: gio::FileMonitor,

    transaction_backend: Box<dyn TransactionBackend>,
    broadcast: BroadcastChannel<BackendMessage>,
}

impl FlatpakBackend {
    pub fn new() -> Rc<Self> {
        let system_installation =
            flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();

        // TODO: Add support for user installations

        let broadcast = BroadcastChannel::new();

        let transaction_backend = if Self::is_sandboxed() {
            Box::new(SandboxBackend::new())
        } else {
            unimplemented!("Host backend not implemented yet");
        };

        let system_monitor = system_installation
            .create_monitor(Some(&gio::Cancellable::new()))
            .unwrap();

        let backend = Rc::new(Self {
            system_installation,
            system_monitor,
            transaction_backend,
            broadcast,
        });

        package_database::init(backend.clone());

        backend.clone().setup_signals();
        backend
    }

    fn setup_signals(self: Rc<Self>) {
        self.system_monitor.connect_changed(|_, _, _, _| {
            debug!("Detected change on system installation.");
        });
    }

    pub fn get_channel(self: Rc<Self>) -> BroadcastChannel<BackendMessage> {
        self.broadcast.clone()
    }

    pub fn get_installed_refs(self: Rc<Self>) -> Vec<InstalledRef> {
        let mut system_refs = self
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        let mut installed_refs = Vec::new();
        installed_refs.append(&mut system_refs);

        installed_refs
    }

    pub fn get_installed_packages(self: Rc<Self>, level: DisplayLevel) -> Vec<InstalledPackage> {
        let mut installed_packages = Vec::new();

        for installed_ref in self.get_installed_refs() {
            let package: InstalledPackage = installed_ref.into();

            let insert = match level {
                DisplayLevel::Apps => package.kind() == PackageKind::App,
                DisplayLevel::Runtimes => {
                    package.kind() == PackageKind::Runtime || package.kind() == PackageKind::App
                }
                DisplayLevel::Extensions => true,
            };

            if insert {
                installed_packages.insert(0, package.clone());
            }
        }

        installed_packages
    }

    pub fn is_package_installed(self: Rc<Self>, package: &dyn Package) -> bool {
        self.get_installed_refs()
            .iter()
            .map(|r| r.get_commit().unwrap().to_string())
            .find(|commit| &package.commit() == commit)
            .map_or(false, |_| true)
    }

    pub fn install_package(self: Rc<Self>, package: &dyn Package) {
        let transaction =
            PackageTransaction::new(package.base_package().clone(), PackageAction::Install);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        self.transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn uninstall_package(self: Rc<Self>, package: &dyn Package) {
        let transaction =
            PackageTransaction::new(package.base_package().clone(), PackageAction::Uninstall);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        self.transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn launch_package(self: Rc<Self>, package: &dyn Package) {
        match std::process::Command::new("flatpak-spawn")
            .arg("--host")
            .arg("flatpak")
            .arg("run")
            .arg(package.ref_name())
            .spawn()
        {
            Ok(_) => (),
            Err(err) => warn!(
                "Unable to launch {}: {}",
                package.ref_name(),
                err.to_string()
            ),
        };
    }

    pub fn cancel_package_transaction(self: Rc<Self>, transaction: Arc<PackageTransaction>) {
        self.transaction_backend
            .cancel_package_transaction(transaction);
    }

    pub fn get_active_transaction(
        self: Rc<Self>,
        package: &dyn Package,
    ) -> Option<Arc<PackageTransaction>> {
        self.transaction_backend
            .get_active_transaction(&package.base_package())
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
