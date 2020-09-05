use gtk::prelude::*;

use crate::backend::Package;
use crate::ui::utils;

pub struct ReleasesBox {
    pub widget: gtk::Box,
    package: Package,
    builder: gtk::Builder,
}

impl ReleasesBox {
    pub fn new(package: Package) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/releases_box.ui");
        get_widget!(builder, gtk::Box, releases_box);

        let releases_box = Self {
            widget: releases_box,
            package,
            builder,
        };

        releases_box.setup_signals();
        releases_box.setup_widgets();
        releases_box
    }

    fn setup_signals(&self) {}

    fn setup_widgets(&self) {
        let releases = self.package.component.releases.clone();
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
}
