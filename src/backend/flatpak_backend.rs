use flatpak::{Installation, InstallationExt};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

use std::collections::HashSet;

use crate::backend::transaction_backend::{SandboxBackend, TransactionBackend};
use crate::backend::{SoukInstalledInfo, SoukPackage, SoukRemoteInfo, SoukTransaction};
use crate::db::queries;

pub struct SoukFlatpakBackendPrivate {
    sys_installation: Installation,
    installed_packages: gio::ListStore,

    transaction_backend: Box<dyn TransactionBackend>,

    sys_monitor: gio::FileMonitor,
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
        let sys_installation = flatpak::Installation::new_system(gio::NONE_CANCELLABLE).unwrap();
        let installed_packages = gio::ListStore::new(SoukPackage::static_type());

        let transaction_backend = if SoukFlatpakBackend::is_sandboxed() {
            Box::new(SandboxBackend::new())
        } else {
            unimplemented!("Host backend not implemented yet");
        };

        let sys_monitor = sys_installation
            .create_monitor(gio::NONE_CANCELLABLE)
            .unwrap();

        Self {
            sys_installation,
            installed_packages,
            transaction_backend,
            sys_monitor,
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

        backend.setup_signals();
        backend
    }

    fn setup_signals(&self) {
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);

        self_
            .sys_monitor
            .connect_changed(clone!(@weak self as this => move|_, _, _, _| {
                this.reload_installed_packages_diff();
            }));
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
        self_.sys_installation.clone()
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
        let installed_ref = self_.sys_installation.get_installed_ref(
            package.get_kind().into(),
            &package.get_name(),
            Some(&package.get_arch()),
            Some(&package.get_branch()),
            gio::NONE_CANCELLABLE,
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

    fn get_installed_packages_vec(&self) -> Vec<SoukPackage> {
        let mut result = Vec::new();

        let self_ = SoukFlatpakBackendPrivate::from_instance(self);
        self_.installed_packages.remove_all();

        let system_refs = self_
            .sys_installation
            .list_installed_refs(gio::NONE_CANCELLABLE)
            .unwrap();

        for installed_ref in system_refs {
            let package: SoukPackage = installed_ref.into();
            result.insert(0, package);
        }

        result
    }

    // Reload installed packages partially
    // -> Find out what's the difference and apply those changes to model
    pub fn reload_installed_packages_diff(&self) {
        debug!("Reload installed packages (diff)...");
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);

        let mut before = HashSet::new();
        let mut after = HashSet::new();

        // before
        for x in 0..self_.installed_packages.get_n_items() {
            let package: SoukPackage = self_
                .installed_packages
                .get_object(x)
                .unwrap()
                .downcast()
                .unwrap();
            before.insert(package.get_ref_name());
        }

        // after
        let new_pkgs = self.get_installed_packages_vec();
        for package in new_pkgs {
            after.insert(package.get_ref_name());
        }

        // find out actual difference
        let mut diff = Vec::new();
        let diff_ba = before.difference(&after);
        for d in diff_ba {
            diff.insert(0, d);
        }
        let diff_ab = after.difference(&before);
        for d in diff_ab {
            diff.insert(0, d);
        }

        warn!("diff: {:?}", diff);
    }

    // Completely reload installed packages
    // -> Clearing model and rebuild it
    pub fn reload_installed_packages_full(&self) {
        debug!("Reload installed packages (full)...");
        let self_ = SoukFlatpakBackendPrivate::from_instance(self);

        self_.installed_packages.remove_all();

        let packages = self.get_installed_packages_vec();
        for package in packages {
            self_.installed_packages.append(&package);
        }
    }
}
