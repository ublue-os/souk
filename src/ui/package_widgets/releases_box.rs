use gtk4::prelude::*;

use crate::backend::Package;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::utils;

pub struct ReleasesBox {
    pub widget: gtk4::Box,
    builder: gtk4::Builder,
}

impl PackageWidget for ReleasesBox {
    fn new() -> Self {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/releases_box.ui");
        get_widget!(builder, gtk4::Box, releases_box);

        Self {
            widget: releases_box,
            builder,
        }
    }

    fn set_package(&self, package: &dyn Package) {
        let releases = package.appdata().expect("No appdata available").releases;
        if !releases.is_empty() {
            self.widget.set_visible(true);
            let release = releases[0].clone();

            get_widget!(self.builder, gtk4::Label, date_label);
            get_widget!(self.builder, gtk4::Label, header_label);
            get_widget!(self.builder, gtk4::Label, description_label);

            utils::set_date_label(&date_label, release.date);
            header_label.set_text(&format!("New in Version {}", &release.version));
            utils::set_label_markup_translatable_string(&description_label, release.description);
        } else {
            self.widget.set_visible(false);
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk4::Label, date_label);
        get_widget!(self.builder, gtk4::Label, header_label);
        get_widget!(self.builder, gtk4::Label, description_label);

        date_label.set_text("–");
        header_label.set_text("–");
        description_label.set_text("–");
    }
}
