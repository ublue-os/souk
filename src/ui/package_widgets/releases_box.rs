use std::cell::RefCell;

use gtk::prelude::*;

use crate::backend::SoukPackage;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::release_row::ReleaseRow;

pub struct ReleasesBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    release_row: RefCell<Option<gtk::ListBoxRow>>,
}

impl PackageWidget for ReleasesBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/releases_box.ui");
        get_widget!(builder, gtk::Box, releases_box);

        Self {
            widget: releases_box,
            builder,
            release_row: RefCell::default(),
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
            let release_row = ReleaseRow::new(release);

            get_widget!(self.builder, gtk::ListBox, releases_box_listbox);

            releases_box_listbox.prepend(&release_row.widget);
            self.release_row.replace(Some(release_row.widget));
        } else {
            self.widget.set_visible(false);
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::ListBox, releases_box_listbox);

        if let Some(row) = self.release_row.borrow_mut().take() {
            releases_box_listbox.remove(&row)
        }
    }
}
