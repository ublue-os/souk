use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use gtk::prelude::*;
use gtk::subclass::prelude::{BoxImpl, WidgetImpl};

use std::cell::RefCell;

use crate::backend::SoukPackage;
use crate::ui::utils;

pub struct SoukPackageRowPrivate {
    package: RefCell<Option<SoukPackage>>,
    builder: gtk::Builder,
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

impl ObjectSubclass for SoukPackageRowPrivate {
    const NAME: &'static str = "SoukPackageRow";
    type ParentType = gtk::Box;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    glib_object_subclass!();

    fn new() -> Self {
        let package = RefCell::new(None);
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_row.ui");

        Self { package, builder }
    }
}

impl ObjectImpl for SoukPackageRowPrivate {
    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => {
                let package = value.get().unwrap();
                *self.package.borrow_mut() = package;
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => Ok(self.package.borrow().to_value()),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for SoukPackageRowPrivate {}

impl BoxImpl for SoukPackageRowPrivate {}

glib_wrapper! {
    pub struct SoukPackageRow(
        Object<subclass::simple::InstanceStruct<SoukPackageRowPrivate>,
        subclass::simple::ClassStruct<SoukPackageRowPrivate>>)
        @extends gtk::Widget, gtk::Box;

    match fn {
        get_type => || SoukPackageRowPrivate::get_type().to_glib(),
    }
}

impl SoukPackageRow {
    pub fn new() -> Self {
        let row = glib::Object::new(SoukPackageRow::static_type(), &[])
            .unwrap()
            .downcast::<SoukPackageRow>()
            .unwrap();

        let self_ = SoukPackageRowPrivate::from_instance(&row);
        get_widget!(self_.builder, gtk::Box, package_row);
        row.append(&package_row);

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        self.connect_notify(Some("package"), |this, _| {
            let self_ = SoukPackageRowPrivate::from_instance(this);
            let package = self_.package.borrow().as_ref().unwrap().clone();

            get_widget!(self_.builder, gtk::Label, title_label);
            get_widget!(self_.builder, gtk::Label, summary_label);
            get_widget!(self_.builder, gtk::Image, icon_image);

            // Icon
            utils::set_icon(&package, &icon_image, 64);

            match package.get_appdata() {
                Some(appdata) => {
                    // Title
                    utils::set_label_translatable_string(&title_label, Some(appdata.name.clone()));
                    // Summary
                    utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
                }
                None => {
                    // Fallback to basic information when no appdata available
                    title_label.set_text(&package.get_name());
                    summary_label.set_text(&package.get_branch());
                }
            };
        });
    }

    pub fn get_package(&self) -> Option<SoukPackage> {
        self.get_property("package").unwrap().get().unwrap()
    }
}
