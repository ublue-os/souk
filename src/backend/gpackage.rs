use appstream::Collection;
use appstream::Component;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::{WidgetImpl, WindowImpl};
use libhandy::prelude::*;

use std::cell::RefCell;

use crate::backend::PackageKind;
use crate::database::DbPackage;

#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "GsPackageKind")]
enum GsPackageKind {
    App = 0,
    Runtime = 1,
    Extension = 2,
}

impl Default for GsPackageKind {
    fn default() -> Self {
        GsPackageKind::App
    }
}

#[derive(Default)]
pub struct GsPackagePrivate {
    kind: RefCell<GsPackageKind>,
    name: RefCell<String>,
    arch: RefCell<String>,
    branch: RefCell<String>,
    commit: RefCell<String>,
    remote: RefCell<String>,
    appdata: RefCell<String>,
}

static PROPERTIES: [subclass::Property; 7] = [
    subclass::Property("kind", |kind| {
        glib::ParamSpec::enum_(
            kind,
            "PackageKind",
            "PackageKind",
            GsPackageKind::static_type(),
            GsPackageKind::default() as i32,
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
    subclass::Property("commit", |commit| {
        glib::ParamSpec::string(commit, "Commit", "Commit", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("remote", |remote| {
        glib::ParamSpec::string(remote, "Remote", "Remote", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("appdata", |appdata| {
        glib::ParamSpec::string(
            appdata,
            "AppData",
            "AppData",
            None,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for GsPackagePrivate {
    const NAME: &'static str = "GsPackage";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        Self::default()
    }
}

impl ObjectImpl for GsPackagePrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("kind", ..) => Ok(self.kind.borrow().to_value()),
            subclass::Property("name", ..) => Ok(self.name.borrow().to_value()),
            subclass::Property("arch", ..) => Ok(self.arch.borrow().to_value()),
            subclass::Property("branch", ..) => Ok(self.branch.borrow().to_value()),
            subclass::Property("commit", ..) => Ok(self.commit.borrow().to_value()),
            subclass::Property("remote", ..) => Ok(self.remote.borrow().to_value()),
            subclass::Property("appdata", ..) => Ok(self.appdata.borrow().to_value()),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct GsPackage(
        Object<subclass::simple::InstanceStruct<GsPackagePrivate>,
        subclass::simple::ClassStruct<GsPackagePrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || GsPackagePrivate::get_type().to_glib(),
    }
}

impl GsPackage {
    pub fn new() -> Self {
        let package = glib::Object::new(GsPackage::static_type(), &[])
            .unwrap()
            .downcast::<GsPackage>()
            .unwrap();

        package
    }

    pub fn appdata(&self) -> Option<Component> {
        let xml: String = self
            .get_property("appdata")
            .unwrap()
            .get()
            .unwrap()
            .unwrap();
        serde_json::from_str(&xml).ok()
    }
}

impl From<DbPackage> for GsPackage {
    fn from(db_package: DbPackage) -> Self {
        let package = GsPackage::new();
        let package_priv = GsPackagePrivate::from_instance(&package);

        let kind = match db_package.kind.as_ref() {
            "app" => GsPackageKind::App,
            "runtime" => GsPackageKind::Runtime,
            _ => GsPackageKind::Extension,
        };

        *package_priv.kind.borrow_mut() = kind;
        *package_priv.name.borrow_mut() = db_package.name;
        *package_priv.arch.borrow_mut() = db_package.arch;
        *package_priv.branch.borrow_mut() = db_package.branch;
        *package_priv.commit.borrow_mut() = db_package.commit;
        *package_priv.remote.borrow_mut() = db_package.remote;
        *package_priv.appdata.borrow_mut() = db_package.appdata;

        package
    }
}
