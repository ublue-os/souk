mod installed_info;
pub use installed_info::SoukInstalledInfo;

mod remote_info;
pub use remote_info::SoukRemoteInfo;

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
#[genum(type_name = "SoukPackageKind")]
enum SoukPackageKind {
    App = 0,
    Runtime = 1,
    Extension = 2,
}

impl Default for SoukPackageKind {
    fn default() -> Self {
        SoukPackageKind::App
    }
}

#[derive(Default)]
pub struct SoukPackagePrivate {
    kind: RefCell<SoukPackageKind>,
    name: RefCell<String>,
    arch: RefCell<String>,
    branch: RefCell<String>,
    remote: RefCell<String>,

    remote_info: RefCell<Option<SoukRemoteInfo>>,
    installed_info: RefCell<Option<SoukInstalledInfo>>,
}

static PROPERTIES: [subclass::Property; 7] = [
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
            glib::ParamFlags::READWRITE,
        )
    }),
    subclass::Property("installed_info", |installed_info| {
        glib::ParamSpec::object(
            installed_info,
            "Installed Information",
            "Installed Information",
            SoukInstalledInfo::static_type(),
            glib::ParamFlags::READWRITE,
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
        Self::default()
    }
}

impl ObjectImpl for SoukPackagePrivate {
    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("remote_info", ..) => {
                let remote_info = value.get().unwrap();
                *self.remote_info.borrow_mut() = remote_info;
            }
            subclass::Property("installed_info", ..) => {
                let installed_info = value.get().unwrap();
                *self.installed_info.borrow_mut() = installed_info;
            }
            _ => unimplemented!(),
        }
    }

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
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukPackage(
        Object<subclass::simple::InstanceStruct<SoukPackagePrivate>,
        subclass::simple::ClassStruct<SoukPackagePrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukPackagePrivate::get_type().to_glib(),
    }
}

impl SoukPackage {
    pub fn new() -> Self {
        let package = glib::Object::new(SoukPackage::static_type(), &[])
            .unwrap()
            .downcast::<SoukPackage>()
            .unwrap();

        package
    }

    pub fn appdata(&self) -> Option<Component> {
        /*let xml: String = self
            .get_property("appdata")
            .unwrap()
            .get()
            .unwrap()
            .unwrap();
        serde_json::from_str(&xml).ok()*/
        None
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

        package
    }
}
