use broadcaster::BroadcastChannel;
use flatpak::{Installation, InstallationExt, InstalledRef, RefExt};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::sync::Arc;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{BackendMessage, Package, PackageAction, PackageTransaction, SoukPackage};
use crate::database::package_database;

pub struct SoukFlatpakBackendPrivate {
    system_installation: Installation,
    installed_packages: gio::ListStore,

    transaction_backend: Box<dyn TransactionBackend>,
    broadcast: BroadcastChannel<BackendMessage>,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property(
    "installed_packages",
    |installed_packages| {
        glib::ParamSpec::object(
            installed_packages,
            "Installed Packages",
            "Installed Packages",
            gio::ListStore::static_type(),
            glib::ParamFlags::READABLE,
        )
    },
)];

impl ObjectSubclass for SoukFlatpakBackendPrivate {
    const NAME: &'static str = "SoukFlatpakBackend";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    glib_object_subclass!();

    fn new() -> Self {
        // TODO: Add support for user installations
        let system_installation =
            flatpak::Installation::new_system(None::<&gio::Cancellable>).unwrap();
        let installed_packages = gio::ListStore::new(SoukPackage::static_type());

        let transaction_backend = if SoukFlatpakBackend::is_sandboxed() {
            Box::new(SandboxBackend::new())
        } else {
            unimplemented!("Host backend not implemented yet");
        };
        let broadcast = BroadcastChannel::new();

        Self {
            system_installation,
            installed_packages,
            transaction_backend,
            broadcast,
        }
    }
}

impl ObjectImpl for SoukFlatpakBackendPrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("installed_packages", ..) => Ok(self.installed_packages.to_value()),
            _ => unimplemented!(),
        }
    }
}

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

    pub fn get_installed_packages(&self) -> gio::ListStore {
        self.reload_installed_packages();
        self.get_property("installed_packages")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_system_installation(&self) -> flatpak::Installation {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.system_installation.clone()
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

    fn is_sandboxed() -> bool {
        std::path::Path::new("/.flatpak-info").exists()
    }

    fn reload_installed_packages(&self) {
        // TODO: This is eating much time on startup. Make this async.

        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.installed_packages.remove_all();

        let mut system_refs = self_
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        for installed_ref in system_refs {
            let package: SoukPackage = installed_ref.into();
            self_.installed_packages.append(&package);
        }
    }
}
