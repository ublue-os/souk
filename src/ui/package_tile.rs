use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};

use std::cell::RefCell;

use crate::backend::SoukPackage;
use crate::ui::utils;

#[derive(Debug, CompositeTemplate)]
pub struct SoukPackageTilePrivate {
    #[template_child(id = "title_label")]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child(id = "summary_label")]
    pub summary_label: TemplateChild<gtk::Label>,
    #[template_child(id = "icon_image")]
    pub icon_image: TemplateChild<gtk::Image>,

    package: RefCell<Option<SoukPackage>>,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property("package", |package| {
    glib::ParamSpec::object(
        package,
        "Package",
        "Package",
        SoukPackage::static_type(),
        glib::ParamFlags::READWRITE,
    )
})];

impl ObjectSubclass for SoukPackageTilePrivate {
    const NAME: &'static str = "SoukPackageTile";
    type Type = SoukPackageTile;
    type ParentType = gtk::FlowBoxChild;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
        klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/package_tile.ui");
        Self::bind_template_children(klass);
    }

    glib::object_subclass!();

    fn new() -> Self {
        let package = RefCell::new(None);
        Self {
            title_label: TemplateChild::default(),
            summary_label: TemplateChild::default(),
            icon_image: TemplateChild::default(),
            package,
        }
    }

    fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
        obj.init_template();
    }
}

impl ObjectImpl for SoukPackageTilePrivate {
    fn set_property(&self, _obj: &SoukPackageTile, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => {
                let package = value.get().unwrap();
                *self.package.borrow_mut() = package;
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &SoukPackageTile, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => self.package.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for SoukPackageTilePrivate {}

impl FlowBoxChildImpl for SoukPackageTilePrivate {}

glib::wrapper! {
    pub struct SoukPackageTile(ObjectSubclass<SoukPackageTilePrivate>)
    @extends gtk::Widget, gtk::FlowBoxChild;
}

impl SoukPackageTile {
    pub fn new() -> Self {
        let tile = glib::Object::new::<Self>(&[]).unwrap();

        tile.setup_signals();
        tile
    }

    fn setup_signals(&self) {
        self.connect_notify(Some("package"), |this, _| {
            let self_ = SoukPackageTilePrivate::from_instance(this);
            let package = self_.package.borrow().as_ref().unwrap().clone();

            // Icon
            utils::set_icon(&package, &self_.icon_image, 64);

            match package.get_appdata() {
                Some(appdata) => {
                    // Title
                    utils::set_label_translatable_string(
                        &self_.title_label,
                        Some(appdata.name.clone()),
                    );
                    // Summary
                    utils::set_label_translatable_string(
                        &self_.summary_label,
                        appdata.summary.clone(),
                    );
                }
                None => {
                    // Fallback to basic information when no appdata available
                    self_.title_label.set_text(&package.get_name());
                    self_.summary_label.set_text(&package.get_branch());
                }
            };
        });
    }

    pub fn set_package(&self, package: &SoukPackage) {
        self.set_property("package", package).unwrap();
    }

    pub fn get_package(&self) -> Option<SoukPackage> {
        self.get_property("package").unwrap().get().unwrap()
    }
}
