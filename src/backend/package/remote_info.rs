use appstream::Component;
use flatpak::prelude::*;
use flatpak::RemoteRef;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;

use crate::database::DbPackage;

#[derive(Default)]
pub struct SoukRemoteInfoPrivate {
    appdata: RefCell<String>,
    commit: RefCell<String>,
    installed_size: RefCell<u64>,
    download_size: RefCell<u64>,
}

static PROPERTIES: [subclass::Property; 3] = [
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
    subclass::Property("download_size", |download_size| {
        glib::ParamSpec::uint64(
            download_size,
            "Download Size",
            "Download Size",
            0,
            std::u64::MAX,
            0,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukRemoteInfoPrivate {
    const NAME: &'static str = "SoukRemoteInfo";
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

impl ObjectImpl for SoukRemoteInfoPrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("commit", ..) => Ok(self.commit.borrow().to_value()),
            subclass::Property("installed_size", ..) => Ok(self.installed_size.borrow().to_value()),
            subclass::Property("download_size", ..) => Ok(self.download_size.borrow().to_value()),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukRemoteInfo(
        Object<subclass::simple::InstanceStruct<SoukRemoteInfoPrivate>,
        subclass::simple::ClassStruct<SoukRemoteInfoPrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukRemoteInfoPrivate::get_type().to_glib(),
    }
}

#[allow(dead_code)]
impl SoukRemoteInfo {
    pub fn new(db_package: &DbPackage) -> Self {
        let info = glib::Object::new(SoukRemoteInfo::static_type(), &[])
            .unwrap()
            .downcast::<SoukRemoteInfo>()
            .unwrap();

        let info_priv = SoukRemoteInfoPrivate::from_instance(&info);
        *info_priv.appdata.borrow_mut() = db_package.appdata.clone();
        *info_priv.commit.borrow_mut() = db_package.commit.clone();
        *info_priv.installed_size.borrow_mut() = db_package.installed_size.clone() as u64;
        *info_priv.download_size.borrow_mut() = db_package.download_size.clone() as u64;

        info
    }

    pub fn new_from_remote_ref(remote_ref: RemoteRef, appdata: String) -> Self {
        let info = glib::Object::new(SoukRemoteInfo::static_type(), &[])
            .unwrap()
            .downcast::<SoukRemoteInfo>()
            .unwrap();

        let info_priv = SoukRemoteInfoPrivate::from_instance(&info);
        *info_priv.appdata.borrow_mut() = appdata;
        *info_priv.commit.borrow_mut() = remote_ref.get_commit().unwrap().to_string();
        *info_priv.installed_size.borrow_mut() = remote_ref.get_installed_size();
        *info_priv.download_size.borrow_mut() = remote_ref.get_download_size();

        info
    }

    pub fn get_commit(&self) -> String {
        self.get_property("commit").unwrap().get().unwrap().unwrap()
    }

    pub fn get_installed_size(&self) -> u64 {
        self.get_property("installed_size")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_download_size(&self) -> u64 {
        self.get_property("download_size")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_appdata(&self) -> Option<Component> {
        let self_ = SoukRemoteInfoPrivate::from_instance(self);
        serde_json::from_str(&self_.appdata.borrow()).ok()
    }
}
