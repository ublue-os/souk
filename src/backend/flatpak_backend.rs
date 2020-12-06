use flatpak::{Installation, InstallationExt};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{SoukInstalledInfo, SoukPackage, SoukRemoteInfo, SoukTransaction};
use crate::db::queries;

pub struct SoukFlatpakBackendPrivate {
    system_installation: Installation,
    installed_packages: gio::ListStore,

    transaction_backend: Box<dyn TransactionBackend>,
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
    type Type = SoukFlatpakBackend;
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
        klass.add_signal(
            "new_transaction",
            glib::SignalFlags::ACTION,
            &[glib::Type::BaseObject],
            glib::Type::Unit,
        );
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

        Self {
            system_installation,
            installed_packages,
            transaction_backend,
        }
    }
}

impl ObjectImpl for SoukFlatpakBackendPrivate {
    fn get_property(&self, _obj: &SoukFlatpakBackend, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("installed_packages", ..) => self.installed_packages.to_value(),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukFlatpakBackend(ObjectSubclass<SoukFlatpakBackendPrivate>);
}

impl SoukFlatpakBackend {
    pub fn new() -> Self {
        let backend = glib::Object::new(SoukFlatpakBackend::static_type(), &[])
            .unwrap()
            .downcast::<SoukFlatpakBackend>()
            .unwrap();

        backend
    }

    pub fn get_installed_packages(&self) -> gio::ListStore {
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

    pub fn launch_package(&self, package: &SoukPackage) {
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

    pub fn add_transaction(&self, transaction: SoukTransaction) {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self.emit("new_transaction", &[&transaction]).unwrap();
        self_.transaction_backend.add_transaction(transaction);
    }

    pub fn cancel_transaction(&self, transaction: SoukTransaction) {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.transaction_backend.cancel_transaction(transaction);
    }

    pub fn get_installed_info(&self, package: &SoukPackage) -> Option<SoukInstalledInfo> {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        let installed_ref = self_.system_installation.get_installed_ref(
            package.get_kind().into(),
            &package.get_name(),
            Some(&package.get_arch()),
            Some(&package.get_branch()),
            None::<&gio::Cancellable>,
        );

        match installed_ref {
            Ok(installed_ref) => Some(SoukInstalledInfo::new(&installed_ref)),
            Err(_) => None,
        }
    }

    pub fn get_remote_info(&self, package: &SoukPackage) -> Option<SoukRemoteInfo> {
        match queries::get_db_package(
            package.get_name(),
            package.get_branch(),
            package.get_remote(),
        )
        .unwrap()
        {
            Some(db_package) => Some(SoukRemoteInfo::new(&db_package)),
            None => None,
        }
    }

    fn is_sandboxed() -> bool {
        std::path::Path::new("/.flatpak-info").exists()
    }

    pub fn reload_installed_packages(&self) {
        debug!("Reload installed packages...");
        // TODO: This is eating much time on startup. Make this async.

        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.installed_packages.remove_all();

        let system_refs = self_
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        for installed_ref in system_refs {
            let package: SoukPackage = installed_ref.into();
            self_.installed_packages.append(&package);
        }
    }
}
