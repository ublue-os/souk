use appstream::Component;
use flatpak::prelude::*;
use flatpak::InstalledRef;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;

#[derive(Default)]
pub struct SoukInstalledInfoPrivate {
    appdata: RefCell<String>,
    commit: RefCell<String>,
    installed_size: RefCell<u64>,
    deploy_dir: RefCell<String>,
}

static PROPERTIES: [subclass::Property; 4] = [
    subclass::Property("appdata", |appdata| {
        glib::ParamSpec::string(
            appdata,
            "AppData",
            "AppData",
            None,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("commit", |commit| {
        glib::ParamSpec::string(commit, "Commit", "Commit", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("installed_size", |installed_size| {
        glib::ParamSpec::uint64(
            installed_size,
            "Installed Size",
            "Installed Size",
            0,
            std::u64::MAX,
            0,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("deploy_dir", |deploy_dir| {
        glib::ParamSpec::string(
            deploy_dir,
            "Deploy Directory",
            "Deploy Directory",
            None,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukInstalledInfoPrivate {
    const NAME: &'static str = "SoukInstalledInfo";
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

impl ObjectImpl for SoukInstalledInfoPrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("appdata", ..) => Ok(self.appdata.borrow().to_value()),
            subclass::Property("commit", ..) => Ok(self.commit.borrow().to_value()),
            subclass::Property("installed_size", ..) => Ok(self.installed_size.borrow().to_value()),
            subclass::Property("deploy_dir", ..) => Ok(self.deploy_dir.borrow().to_value()),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukInstalledInfo(
        Object<subclass::simple::InstanceStruct<SoukInstalledInfoPrivate>,
        subclass::simple::ClassStruct<SoukInstalledInfoPrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukInstalledInfoPrivate::get_type().to_glib(),
    }
}

impl SoukInstalledInfo {
    pub fn new(installed_ref: &InstalledRef) -> Self {
        let info = glib::Object::new(SoukInstalledInfo::static_type(), &[])
            .unwrap()
            .downcast::<SoukInstalledInfo>()
            .unwrap();

        let info_priv = SoukInstalledInfoPrivate::from_instance(&info);
        *info_priv.commit.borrow_mut() = installed_ref.get_latest_commit().unwrap().to_string();
        *info_priv.installed_size.borrow_mut() = installed_ref.get_installed_size();
        *info_priv.deploy_dir.borrow_mut() = installed_ref.get_deploy_dir().unwrap().to_string();

        info
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
