use std::cell::RefCell;

use glib::SignalHandlerId;
use gtk::prelude::*;

use crate::backend::SoukPackage;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::release_row::ReleaseRow;
use crate::ui::releases_window::ReleasesWindow;

pub struct ReleasesBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    signal_handler_id: RefCell<Option<SignalHandlerId>>,
    release_row: RefCell<Option<gtk::ListBoxRow>>,
}

impl PackageWidget for ReleasesBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/releases_box.ui");
        get_widget!(builder, gtk::Box, releases_box);

        Self {
            widget: releases_box,
            builder,
            signal_handler_id: RefCell::default(),
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

            self.signal_handler_id
                .replace(Some(releases_box_listbox.connect_row_activated(
                    move |_row, _gdata| {
                        ReleasesWindow::new(releases.clone()).widget.present();
                    },
                )));
            releases_box_listbox.prepend(&release_row.widget);
            self.release_row.replace(Some(release_row.widget));
        } else {
            self.widget.set_visible(false);
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::ListBox, releases_box_listbox);

        if let Some(id) = self.signal_handler_id.borrow_mut().take() {
            releases_box_listbox.disconnect(id);
        }
        if let Some(row) = self.release_row.borrow_mut().take() {
            releases_box_listbox.remove(&row)
        }
    }
}
