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

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::SoukApplication;
use crate::backend::{
    SoukFlatpakBackend, SoukTransaction, SoukTransactionMode, SoukTransactionState,
};
use crate::db::DbPackage;

pub struct SoukPackagePrivate {
    kind: RefCell<SoukPackageKind>,
    name: RefCell<String>,
    arch: RefCell<String>,
    branch: RefCell<String>,
    remote: RefCell<String>,

    remote_info: RefCell<Option<SoukRemoteInfo>>,
    installed_info: RefCell<Option<SoukInstalledInfo>>,

    transaction: RefCell<SoukTransaction>,
    transaction_action: RefCell<SoukPackageAction>,
    transaction_state: RefCell<SoukTransactionState>,

    flatpak_backend: SoukFlatpakBackend,
    fb_transaction_id: RefCell<Option<glib::SignalHandlerId>>,
    fb_pkg_changes_id: RefCell<Option<glib::SignalHandlerId>>,
}

static PROPERTIES: [subclass::Property; 10] = [
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
    subclass::Property("is_installed", |remote| {
        glib::ParamSpec::boolean(
            remote,
            "Is Installed",
            "Is Installed",
            false,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukPackagePrivate {
    const NAME: &'static str = "SoukPackage";
    type Type = SoukPackage;
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
            fb_transaction_id: RefCell::default(),
            fb_pkg_changes_id: RefCell::default(),
        }
    }
}

impl ObjectImpl for SoukPackagePrivate {
    fn get_property(&self, _obj: &SoukPackage, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("kind", ..) => self.kind.borrow().to_value(),
            subclass::Property("name", ..) => self.name.borrow().to_value(),
            subclass::Property("arch", ..) => self.arch.borrow().to_value(),
            subclass::Property("branch", ..) => self.branch.borrow().to_value(),
            subclass::Property("remote", ..) => self.remote.borrow().to_value(),
            subclass::Property("remote_info", ..) => self.remote_info.borrow().to_value(),
            subclass::Property("installed_info", ..) => self.installed_info.borrow().to_value(),
            subclass::Property("transaction_action", ..) => {
                self.transaction_action.borrow().to_value()
            }
            subclass::Property("transaction_state", ..) => {
                self.transaction_state.borrow().to_value()
            }
            subclass::Property("is_installed", ..) => {
                self.installed_info.borrow().is_some().to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl Drop for SoukPackagePrivate {
    fn drop(&mut self) {
        // We need to disconnect manually the signal again,
        // otherwise this object never would get dropped,
        // since `flatpak_backend` still would hold a reference of it.
        //
        // Normally we should bind the signal by using
        // g_signal_connect_object
        // to avoid this problem, but there aren't bindings for it available yet.
        // https://github.com/gtk-rs/gtk-rs/issues/64

        let fb_transaction_id = self.fb_transaction_id.borrow_mut().take();
        self.flatpak_backend.disconnect(fb_transaction_id.unwrap());

        let fb_pkg_changes_id = self.fb_pkg_changes_id.borrow_mut().take();
        self.flatpak_backend.disconnect(fb_pkg_changes_id.unwrap());
    }
}

glib_wrapper! {
    pub struct SoukPackage(ObjectSubclass<SoukPackagePrivate>);
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

        // Check for new transactions from SoukFlatpakBackend side...
        let fb_transaction_id = self_
            .flatpak_backend
            .connect_local(
                "new_transaction",
                false,
                clone!(@weak self as this => @default-panic, move |data|{
                    let object: glib::Object = data[1].get().unwrap().unwrap();
                    let transaction: SoukTransaction = object.downcast().unwrap();

                    // Check if this package is affected by this transaction
                    if transaction.get_package().get_ref_name() == this.get_ref_name(){
                        let self_ = SoukPackagePrivate::from_instance(&this);

                        // Set `transaction_action` property
                        // We're using `transaction_action` and `transaction_state` as own property
                        // and not the actual transaction itself
                        *self_.transaction_action.borrow_mut() = transaction.get_action();
                        this.notify("transaction_action");

                        // Listen to transaction state changes
                        *self_.transaction.borrow_mut() = transaction.clone();
                        this.connect_transaction_state_changes();
                    }

                    None
                }),
            )
            .unwrap();
        *self_.fb_transaction_id.borrow_mut() = Some(fb_transaction_id);

        // Check for package changes SoukFlatpakBackend side, so this package can updated if needed
        let fb_pkg_changes_id = self_
            .flatpak_backend
            .connect_local(
                "package_change",
                false,
                clone!(@weak self as this => @default-panic, move |data|{
                    let action: SoukPackageAction = data[1].get().unwrap().unwrap();
                    let ref_name: String = data[2].get().unwrap().unwrap();

                    // Check if this package is affected by this change
                    if ref_name == this.get_ref_name(){
                        this.update_installed_info();
                    }

                    None
                }),
            )
            .unwrap();
        *self_.fb_pkg_changes_id.borrow_mut() = Some(fb_pkg_changes_id);
    }

    fn connect_transaction_state_changes(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);

        // We need the signal handler id to disconnect from the signal later when the transaction finished
        let signal_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));

        let transaction = self_.transaction.borrow().clone();
        *signal_id.borrow_mut() = Some(transaction.connect_local("notify::state", false, clone!(@weak self as this, @strong signal_id => @default-return None, move |data|{
            let self_ = SoukPackagePrivate::from_instance(&this);

            let object: glib::Object = data[0].get().unwrap().unwrap();
            let transaction: SoukTransaction = object.downcast().unwrap();
            let state = transaction.get_state();

            // Update `transaction_state` property of package when transaction is still active
            if state.get_mode() == SoukTransactionMode::Running || state.get_mode() == SoukTransactionMode::Waiting {
                *self_.transaction_state.borrow_mut() = state;
                this.notify("transaction_state");
            } else {
                // The transaction isn't running anymore, so reset `transaction_action`
                // and `transaction_state` properties
                *self_.transaction_action.borrow_mut() = SoukPackageAction::default();
                *self_.transaction_state.borrow_mut() = SoukTransactionState::default();
                this.notify("transaction_action");
                this.notify("transaction_state");

                // Disconnect from `notify::state` signal
                transaction.disconnect(signal_id.borrow_mut().take().unwrap());
                *self_.transaction.borrow_mut() = SoukTransaction::default();

                // ... and make sure the installed info is up-to-date
                this.update_installed_info();
            }

            None
        })).unwrap());
    }

    fn update_remote_info(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);
        let remote_info = self_.flatpak_backend.get_remote_info(&self);
        *self_.remote_info.borrow_mut() = remote_info;
        self.notify("remote_info");
    }

    fn update_installed_info(&self) {
        let self_ = SoukPackagePrivate::from_instance(self);
        let installed_info = self_.flatpak_backend.get_installed_info(&self);
        *self_.installed_info.borrow_mut() = installed_info;
        self.notify("installed_info");
        self.notify("is_installed");
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

        let transaction = self_.transaction.borrow().clone();
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

    // TODO: This is currently not needed, but I have the feeling
    // it's going to be useful later...
    pub fn get_transaction_action(&self) -> SoukPackageAction {
        self.get_property("transaction_action")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_transaction_state(&self) -> SoukTransactionState {
        self.get_property("transaction_state")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_is_installed(&self) -> bool {
        self.get_property("is_installed")
            .unwrap()
            .get()
            .unwrap()
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
        let keyfile_bytes = installed_ref.load_metadata(gio::NONE_CANCELLABLE).unwrap();
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

        // Set installed info
        let installed_info = SoukInstalledInfo::new(&installed_ref);
        *package_priv.installed_info.borrow_mut() = Some(installed_info);

        // Set remote info
        package.update_remote_info();

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

        // Set remote info
        let remote_info = SoukRemoteInfo::new_from_remote_ref(remote_ref.0, remote_ref.1);
        *package_priv.remote_info.borrow_mut() = Some(remote_info);

        // Set installed info
        package.update_installed_info();

        package
    }
}
