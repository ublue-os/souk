use appstream::{Collection, Component};
use flatpak::prelude::*;
use flatpak::InstalledRef;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Default)]
pub struct SoukInstalledInfoPrivate {
    commit: RefCell<String>,
    installed_size: RefCell<u64>,
    deploy_dir: RefCell<String>,

    name: RefCell<String>,
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
    type Type = SoukInstalledInfo;
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
    fn get_property(&self, _obj: &SoukInstalledInfo, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("commit", ..) => self.commit.borrow().to_value(),
            subclass::Property("installed_size", ..) => self.installed_size.borrow().to_value(),
            subclass::Property("deploy_dir", ..) => self.deploy_dir.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukInstalledInfo(ObjectSubclass<SoukInstalledInfoPrivate>);
}

#[allow(dead_code)]
impl SoukInstalledInfo {
    pub fn new(installed_ref: &InstalledRef) -> Self {
        let info = glib::Object::new(SoukInstalledInfo::static_type(), &[])
            .unwrap()
            .downcast::<SoukInstalledInfo>()
            .unwrap();

        let info_priv = SoukInstalledInfoPrivate::from_instance(&info);
        *info_priv.commit.borrow_mut() = installed_ref.get_commit().unwrap().to_string();
        *info_priv.installed_size.borrow_mut() = installed_ref.get_installed_size();
        *info_priv.deploy_dir.borrow_mut() = installed_ref.get_deploy_dir().unwrap().to_string();
        *info_priv.name.borrow_mut() = installed_ref.get_name().unwrap().to_string();

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

    pub fn get_deploy_dir(&self) -> String {
        self.get_property("deploy_dir")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_appdata(&self) -> Option<Component> {
        let self_ = SoukInstalledInfoPrivate::from_instance(self);

        // Load appdata
        let mut path = PathBuf::new();
        let appstream_dir = self.get_deploy_dir();
        path.push(appstream_dir);
        path.push(&format!(
            "files/share/app-info/xmls/{}.xml.gz",
            self_.name.borrow().to_string()
        ));

        // Parse appstream data
        Collection::from_gzipped(path.clone())
            .map(|appdata| appdata.components[0].clone())
            .ok()
    }
}
