use broadcaster::BroadcastChannel;
use flatpak::{Installation, InstallationExt, InstalledRef, RefExt};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{
    BackendMessage, InstalledPackage, Package, PackageAction, PackageKind, PackageTransaction,
};
use crate::database::{package_database, DisplayLevel};

pub struct SoukFlatpakBackendPrivate {
    pub system_installation: Installation,

    transaction_backend: Box<dyn TransactionBackend>,
    broadcast: BroadcastChannel<BackendMessage>,
}

impl ObjectSubclass for SoukFlatpakBackendPrivate {
    const NAME: &'static str = "SoukFlatpakBackend";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        // TODO: Add support for user installations
        let system_installation =
            flatpak::Installation::new_system(None::<&gio::Cancellable>).unwrap();
        let broadcast = BroadcastChannel::new();

        let transaction_backend = if SoukFlatpakBackend::is_sandboxed() {
            Box::new(SandboxBackend::new())
        } else {
            unimplemented!("Host backend not implemented yet");
        };

        Self {
            system_installation,
            transaction_backend,
            broadcast,
        }
    }
}

impl ObjectImpl for SoukFlatpakBackendPrivate {}

glib_wrapper! {
    pub struct SoukFlatpakBackend(
        Object<subclass::simple::InstanceStruct<SoukFlatpakBackendPrivate>,
        subclass::simple::ClassStruct<SoukFlatpakBackendPrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukFlatpakBackendPrivate::get_type().to_glib(),
    }
}

impl SoukFlatpakBackend {
    pub fn new() -> Self {
        let backend = glib::Object::new(SoukFlatpakBackend::static_type(), &[])
            .unwrap()
            .downcast::<SoukFlatpakBackend>()
            .unwrap();

        package_database::init(backend.clone());

        backend
    }

    pub fn get_system_installation(&self) -> flatpak::Installation {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.system_installation.clone()
    }

    pub fn get_channel(&self) -> BroadcastChannel<BackendMessage> {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.broadcast.clone()
    }

    pub fn get_installed_refs(&self) -> Vec<InstalledRef> {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);

        let mut system_refs = self_
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        let mut installed_refs = Vec::new();
        installed_refs.append(&mut system_refs);

        installed_refs
    }

    pub fn get_installed_packages(&self, level: DisplayLevel) -> Vec<InstalledPackage> {
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

    pub fn is_package_installed(&self, package: &dyn Package) -> bool {
        self.get_installed_refs()
            .iter()
            .map(|r| r.get_commit().unwrap().to_string())
            .find(|commit| &package.commit() == commit)
            .map_or(false, |_| true)
    }

    pub fn install_package(&self, package: &dyn Package) {
        let transaction =
            PackageTransaction::new(package.base_package().clone(), PackageAction::Install);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_
            .transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn uninstall_package(&self, package: &dyn Package) {
        let transaction =
            PackageTransaction::new(package.base_package().clone(), PackageAction::Uninstall);
        self.clone()
            .send_message(BackendMessage::PackageTransaction(transaction.clone()));

        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_
            .transaction_backend
            .add_package_transaction(transaction);
    }

    pub fn launch_package(&self, package: &dyn Package) {
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

    pub fn cancel_package_transaction(&self, transaction: Arc<PackageTransaction>) {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_
            .transaction_backend
            .cancel_package_transaction(transaction);
    }

    pub fn get_active_transaction(&self, package: &dyn Package) -> Option<Arc<PackageTransaction>> {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_
            .transaction_backend
            .get_active_transaction(&package.base_package())
    }

    fn send_message(&self, message: BackendMessage) {
        // TODO: Port this to a gtk-rs signal / callback
        //let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        //let future = async move {
        //    self_.broadcast.send(&message).await.unwrap();
        //};
        //spawn!(future);
    }

    fn is_sandboxed() -> bool {
        std::path::Path::new("/.flatpak-info").exists()
    }
}
