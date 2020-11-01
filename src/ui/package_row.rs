use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::{BoxImpl, WidgetImpl};
use libhandy::prelude::*;

use std::cell::RefCell;

use crate::app::{Action, SoukApplication, SoukApplicationPrivate};
use crate::backend::SoukPackage;
use crate::config;
use crate::ui::utils;

pub struct SoukPackageRowPrivate {
    builder: gtk::Builder,
}

impl ObjectSubclass for SoukPackageRowPrivate {
    const NAME: &'static str = "SoukPackageRow";
    type ParentType = gtk::Box;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_row.ui");

        Self { builder }
    }
}

impl ObjectImpl for SoukPackageRowPrivate {}

impl WidgetImpl for SoukPackageRowPrivate {}

impl BoxImpl for SoukPackageRowPrivate {}

glib_wrapper! {
    pub struct SoukPackageRow(
        Object<subclass::simple::InstanceStruct<SoukPackageRowPrivate>,
        subclass::simple::ClassStruct<SoukPackageRowPrivate>,
        GsApplicationWindowClass>)
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

        row
    }

    pub fn set_package(&self, package: &SoukPackage) {
        let self_ = SoukPackageRowPrivate::from_instance(self);
        get_widget!(self_.builder, gtk::Label, title_label);
        get_widget!(self_.builder, gtk::Label, summary_label);
        get_widget!(self_.builder, gtk::Image, icon_image);

        // Icon
        utils::set_gs_icon(package, &icon_image, 64);

        match package.appdata() {
            Some(appdata) => {
                // Title
                utils::set_label_translatable_string(&title_label, Some(appdata.name.clone()));
                // Summary
                utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
            }
            None => {
                // Fallback to basic information when no appdata available
                title_label.set_text(
                    package
                        .get_property("name")
                        .unwrap()
                        .get()
                        .unwrap()
                        .unwrap(),
                );
                summary_label.set_text(
                    package
                        .get_property("branch")
                        .unwrap()
                        .get()
                        .unwrap()
                        .unwrap(),
                );
            }
        };
    }
}
