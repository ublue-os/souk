use gtk::prelude::*;

use crate::backend::SoukPackage;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::utils;

pub struct ReleasesBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
}

impl PackageWidget for ReleasesBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/releases_box.ui");
        get_widget!(builder, gtk::Box, releases_box);

        Self {
            widget: releases_box,
            builder,
        }
    }

    fn set_package(&self, package: &SoukPackage) {
        let releases = package
            .get_appdata()
            .expect("No appdata available")
            .releases;
        if !releases.is_empty() {
            self.widget.set_visible(true);
            let release = releases[0].clone();

            get_widget!(self.builder, gtk::Label, date_label);
            get_widget!(self.builder, gtk::Label, header_label);
            get_widget!(self.builder, gtk::Label, description_label);

            utils::set_date_label(&date_label, release.date);
            header_label.set_text(&format!("New in Version {}", &release.version));
            utils::set_label_markup_translatable_string(&description_label, release.description);
        } else {
            self.widget.set_visible(false);
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::Label, date_label);
        get_widget!(self.builder, gtk::Label, header_label);
        get_widget!(self.builder, gtk::Label, description_label);

        date_label.set_text("–");
        header_label.set_text("–");
        description_label.set_text("–");
    }
}
