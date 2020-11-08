mod installed_info;
mod package_action;
mod package_kind;
mod remote_info;
pub use installed_info::SoukInstalledInfo;
pub use package_action::SoukPackageAction;
pub use package_kind::SoukPackageKind;
pub use remote_info::SoukRemoteInfo;

use appstream::Component;
use flatpak::prelude::*;
use flatpak::{InstalledRef, RemoteRef};
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::SoukApplication;
use crate::backend::{
    SoukFlatpakBackend, SoukTransaction, SoukTransactionMode, SoukTransactionState,
};
use crate::database::DbPackage;

pub struct SoukPackagePrivate {
    kind: RefCell<SoukPackageKind>,
    name: RefCell<String>,
    arch: RefCell<String>,
    branch: RefCell<String>,
    remote: RefCell<String>,

    remote_info: RefCell<Option<SoukRemoteInfo>>,
    installed_info: RefCell<Option<SoukInstalledInfo>>,

    transaction: RefCell<Option<SoukTransaction>>,
    transaction_action: RefCell<SoukPackageAction>,
    transaction_state: RefCell<Option<SoukTransactionState>>,

    flatpak_backend: SoukFlatpakBackend,
}

static PROPERTIES: [subclass::Property; 9] = [
    subclass::Property("kind", |kind| {
        glib::ParamSpec::enum_(
            kind,
            "PackageKind",
            "PackageKind",
            SoukPackageKind::static_type(),
            SoukPackageKind::default() as i32,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("name", |name| {
        glib::ParamSpec::string(name, "Name", "Name", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("arch", |arch| {
        glib::ParamSpec::string(arch, "Arch", "Arch", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("branch", |branch| {
        glib::ParamSpec::string(branch, "Branch", "Branch", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("remote", |remote| {
        glib::ParamSpec::string(remote, "Remote", "Remote", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("remote_info", |remote_info| {
        glib::ParamSpec::object(
            remote_info,
            "Remote Information",
            "Remote Information",
            SoukRemoteInfo::static_type(),
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("installed_info", |installed_info| {
        glib::ParamSpec::object(
            installed_info,
            "Installed Information",
            "Installed Information",
            SoukInstalledInfo::static_type(),
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("transaction_action", |transaction_action| {
        glib::ParamSpec::enum_(
            transaction_action,
            "Transaction Action",
            "Transaction Action",
            SoukPackageAction::static_type(),
            SoukPackageAction::default() as i32,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("transaction_state", |transaction_state| {
        glib::ParamSpec::object(
            transaction_state,
            "Transaction State",
            "Transaction State",
            SoukTransactionState::static_type(),
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukPackagePrivate {
    const NAME: &'static str = "SoukPackage";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        let app: SoukApplication = gio::Application::get_default().unwrap().downcast().unwrap();
        let flatpak_backend = app.get_flatpak_backend();

        SoukPackagePrivate {
            kind: RefCell::default(),
            name: RefCell::default(),
            arch: RefCell::default(),
            branch: RefCell::default(),
            remote: RefCell::default(),
            remote_info: RefCell::default(),
            installed_info: RefCell::default(),
            transaction: RefCell::default(),
            transaction_action: RefCell::default(),
            transaction_state: RefCell::default(),
            flatpak_backend,
        }
    }
}

impl ObjectImpl for SoukPackagePrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("kind", ..) => Ok(self.kind.borrow().to_value()),
            subclass::Property("name", ..) => Ok(self.name.borrow().to_value()),
            subclass::Property("arch", ..) => Ok(self.arch.borrow().to_value()),
            subclass::Property("branch", ..) => Ok(self.branch.borrow().to_value()),
            subclass::Property("remote", ..) => Ok(self.remote.borrow().to_value()),
            subclass::Property("remote_info", ..) => Ok(self.remote_info.borrow().to_value()),
            subclass::Property("installed_info", ..) => Ok(self.installed_info.borrow().to_value()),
            subclass::Property("transaction_action", ..) => {
                Ok(self.transaction_action.borrow().to_value())
            }
            subclass::Property("transaction_state", ..) => {
                Ok(self.transaction_state.borrow().to_value())
            }
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukPackage(
        Object<subclass::simple::InstanceStruct<SoukPackagePrivate>,
        subclass::simple::ClassStruct<SoukPackagePrivate>>);

    match fn {
        get_type => || SoukPackagePrivate::get_type().to_glib(),
    }
}

#[allow(dead_code)]
impl SoukPackage {
    pub fn new() -> Self {
        let package = glib::Object::new(SoukPackage::static_type(), &[])
            .unwrap()
            .downcast::<SoukPackage>()
            .unwrap();

        package.setup_signals();
        package
    }

    fn setup_signals(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);
        self_
            .flatpak_backend
            .connect_local(
                "new_transaction",
                false,
                clone!(@strong self as this => @default-return None::<glib::Value>, move |data|{
                    let object: glib::Object = data[1].get().unwrap().unwrap();
                    let transaction: SoukTransaction = object.downcast().unwrap();

                    // Check if this package is affected by this transaction
                    if transaction.get_package() == this{
                        // Set transaction action
                        let self_ = SoukPackagePrivate::from_instance(&this);
                        *self_.transaction.borrow_mut() = Some(transaction.clone());

                        *self_.transaction_action.borrow_mut() = transaction.get_action();
                        this.notify("transaction_action");

                        // Listen to transaction state changes
                        // We're showing the transaction state as own property for SoukPackage
                        this.connect_state_changes(transaction);
                    }

                    None
                }),
            )
            .unwrap();
    }

    fn connect_state_changes(&self, transaction: SoukTransaction) {
        let signal_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));
        *signal_id.borrow_mut() = Some(transaction.connect_local("notify::state", false, clone!(@weak self as this, @weak transaction, @strong signal_id => @default-return None, move |data|{
            let object: glib::Object = data[0].get().unwrap().unwrap();
            let transaction: SoukTransaction = object.downcast().unwrap();
            let state = transaction.get_state();

            // Update `transaction_state` property of package when transaction is still active
            if state.get_mode() == SoukTransactionMode::Running || state.get_mode() == SoukTransactionMode::Waiting{
                let self_ = SoukPackagePrivate::from_instance(&this);
                *self_.transaction_state.borrow_mut() = Some(state);
                this.notify("transaction_state");
            }else{
                // When transaction isn't running anymore, reset `transaction_action`...
                let self_ = SoukPackagePrivate::from_instance(&this);
                *self_.transaction_action.borrow_mut() = SoukPackageAction::None;
                this.notify("transaction_action");

                // ... and `transaction_state`
                let self_ = SoukPackagePrivate::from_instance(&this);
                *self_.transaction_state.borrow_mut() = None;
                this.notify("transaction_state");

                // Disconnect from signal
                transaction.disconnect(signal_id.borrow_mut().take().unwrap());

                *self_.transaction.borrow_mut() = None;

                this.update_installed_info();
            }

            None
        })).unwrap());
    }

    fn update_installed_info(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);
        let installed_info = self_.flatpak_backend.get_installed_info(&self);
        *self_.installed_info.borrow_mut() = installed_info;
        self.notify("installed_info");
    }

    pub fn install(&self) {
        let transaction = SoukTransaction::new(self.clone(), SoukPackageAction::Install);
        let self_ = SoukPackagePrivate::from_instance(self);
        self_.flatpak_backend.add_transaction(transaction);
    }

    pub fn uninstall(&self) {
        let transaction = SoukTransaction::new(self.clone(), SoukPackageAction::Uninstall);
        let self_ = SoukPackagePrivate::from_instance(self);
        self_.flatpak_backend.add_transaction(transaction);
    }

    pub fn launch(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);
        self_.flatpak_backend.launch_package(self);
    }

    pub fn cancel_transaction(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);

        let transaction = self_.transaction.borrow().as_ref().unwrap().clone();
        self_.flatpak_backend.cancel_transaction(transaction);
    }

    pub fn get_kind(&self) -> SoukPackageKind {
        self.get_property("kind").unwrap().get().unwrap().unwrap()
    }

    pub fn get_name(&self) -> String {
        self.get_property("name").unwrap().get().unwrap().unwrap()
    }

    pub fn get_arch(&self) -> String {
        self.get_property("arch").unwrap().get().unwrap().unwrap()
    }

    pub fn get_branch(&self) -> String {
        self.get_property("branch").unwrap().get().unwrap().unwrap()
    }

    pub fn get_remote(&self) -> String {
        self.get_property("remote").unwrap().get().unwrap().unwrap()
    }

    pub fn get_remote_info(&self) -> Option<SoukRemoteInfo> {
        self.get_property("remote_info").unwrap().get().unwrap()
    }

    pub fn get_installed_info(&self) -> Option<SoukInstalledInfo> {
        self.get_property("installed_info").unwrap().get().unwrap()
    }

    pub fn get_transaction_action(&self) -> SoukPackageAction {
        self.get_property("transaction_action")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_transaction_state(&self) -> Option<SoukTransactionState> {
        self.get_property("transaction_state")
            .unwrap()
            .get()
            .unwrap()
    }

    pub fn get_appdata(&self) -> Option<Component> {
        let self_ = SoukPackagePrivate::from_instance(self);

        if self_.remote_info.borrow().is_some() {
            self_.remote_info.borrow().as_ref().unwrap().get_appdata()
        } else {
            self_
                .installed_info
                .borrow()
                .as_ref()
                .unwrap()
                .get_appdata()
        }
    }

    pub fn get_ref_name(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            &self.get_kind().to_string(),
            &self.get_name(),
            &self.get_arch(),
            &self.get_branch()
        )
    }
}

impl From<DbPackage> for SoukPackage {
    fn from(db_package: DbPackage) -> Self {
        let package = SoukPackage::new();
        let package_priv = SoukPackagePrivate::from_instance(&package);

        let kind = match db_package.kind.as_ref() {
            "app" => SoukPackageKind::App,
            "runtime" => SoukPackageKind::Runtime,
            _ => SoukPackageKind::Extension,
        };

        *package_priv.kind.borrow_mut() = kind;
        *package_priv.name.borrow_mut() = db_package.name.clone();
        *package_priv.arch.borrow_mut() = db_package.arch.clone();
        *package_priv.branch.borrow_mut() = db_package.branch.clone();
        *package_priv.remote.borrow_mut() = db_package.remote.clone();

        let remote_info = SoukRemoteInfo::new(&db_package);
        *package_priv.remote_info.borrow_mut() = Some(remote_info);

        package.update_installed_info();
        package
    }
}

impl From<InstalledRef> for SoukPackage {
    fn from(installed_ref: InstalledRef) -> Self {
        let keyfile_bytes = installed_ref
            .load_metadata(Some(&gio::Cancellable::new()))
            .unwrap();
        let keyfile = glib::KeyFile::new();
        keyfile
            .load_from_bytes(&keyfile_bytes, glib::KeyFileFlags::NONE)
            .unwrap();

        let package = SoukPackage::new();
        let package_priv = SoukPackagePrivate::from_instance(&package);

        *package_priv.kind.borrow_mut() = SoukPackageKind::from_keyfile(keyfile);
        *package_priv.name.borrow_mut() = installed_ref.get_name().unwrap().to_string();
        *package_priv.arch.borrow_mut() = installed_ref.get_arch().unwrap().to_string();
        *package_priv.branch.borrow_mut() = installed_ref.get_branch().unwrap().to_string();
        *package_priv.remote.borrow_mut() = installed_ref.get_origin().unwrap().to_string();

        let installed_info = SoukInstalledInfo::new(&installed_ref);
        *package_priv.installed_info.borrow_mut() = Some(installed_info);

        package.update_installed_info();
        package
    }
}

impl From<(RemoteRef, String)> for SoukPackage {
    fn from(remote_ref: (RemoteRef, String)) -> Self {
        let keyfile_bytes = remote_ref.0.get_metadata().unwrap();
        let keyfile = glib::KeyFile::new();
        keyfile
            .load_from_bytes(&keyfile_bytes, glib::KeyFileFlags::NONE)
            .unwrap();

        let package = SoukPackage::new();
        let package_priv = SoukPackagePrivate::from_instance(&package);

        *package_priv.kind.borrow_mut() = SoukPackageKind::from_keyfile(keyfile);
        *package_priv.name.borrow_mut() = remote_ref.0.get_name().unwrap().to_string();
        *package_priv.arch.borrow_mut() = remote_ref.0.get_arch().unwrap().to_string();
        *package_priv.branch.borrow_mut() = remote_ref.0.get_branch().unwrap().to_string();
        *package_priv.remote.borrow_mut() = remote_ref.0.get_remote_name().unwrap().to_string();

        let remote_info = SoukRemoteInfo::new_from_remote_ref(remote_ref.0, remote_ref.1);
        *package_priv.remote_info.borrow_mut() = Some(remote_info);

        package
    }
}
