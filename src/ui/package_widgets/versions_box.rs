use std::cell::RefCell;

use glib::SignalHandlerId;
use gtk::prelude::*;

use crate::backend::SoukPackage;
use crate::ui::package_widgets::PackageWidget;
use crate::ui::version_row::VersionRow;
use crate::ui::versions_window::VersionsWindow;

pub struct VersionsBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    signal_handler_id: RefCell<Option<SignalHandlerId>>,
    version_row: RefCell<Option<gtk::ListBoxRow>>,
}

impl PackageWidget for VersionsBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/versions_box.ui");
        get_widget!(builder, gtk::Box, versions_box);

        Self {
            widget: versions_box,
            builder,
            signal_handler_id: RefCell::default(),
            version_row: RefCell::default(),
        }
    }

    fn set_package(&self, package: &SoukPackage) {
        let versions = package
            .get_appdata()
            .expect("No appdata available")
            .releases;
        if !versions.is_empty() {
            self.widget.set_visible(true);
            let version = versions[0].clone();
            let version_row = VersionRow::new(version, true);

            get_widget!(self.builder, gtk::ListBox, versions_box_listbox);
            get_widget!(self.builder, gtk::ListBoxRow, version_history_row);

            version_history_row.set_activatable(versions.len() > 1);
            version_history_row.set_sensitive(versions.len() > 1);

            self.signal_handler_id
                .replace(Some(versions_box_listbox.connect_row_activated(
                    move |_row, _gdata| {
                        VersionsWindow::new(versions.clone()).widget.present();
                    },
                )));
            versions_box_listbox.prepend(&version_row.widget);
            self.version_row.replace(Some(version_row.widget));
        } else {
            self.widget.set_visible(false);
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::ListBox, versions_box_listbox);

        if let Some(id) = self.signal_handler_id.borrow_mut().take() {
            versions_box_listbox.disconnect(id);
        }
        if let Some(row) = self.version_row.borrow_mut().take() {
            versions_box_listbox.remove(&row)
        }
    }
}
