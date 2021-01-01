use appstream::Release;
use gtk::prelude::*;

use crate::ui::utils;

pub struct VersionRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    release: Release,
}

impl VersionRow {
    pub fn new(release: Release, show_new_header: bool) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/version_row.ui");
        get_widget!(builder, gtk::ListBoxRow, version_row);

        let versions_window_row = Self {
            widget: version_row,
            builder,
            release,
        };
        versions_window_row.setup_widgets(show_new_header);
        versions_window_row
    }

    fn setup_widgets(&self, show_new_header: bool) {
        let release = self.release.clone();
        self.widget.set_visible(true);

        get_widget!(self.builder, gtk::Label, date_label);
        get_widget!(self.builder, gtk::Label, header_label);
        get_widget!(self.builder, gtk::Box, description_box);

        utils::set_date_label(&date_label, release.date);
        if show_new_header {
            header_label.set_text(&format!("New in Version {}", &release.version));
        } else {
            header_label.set_text(&format!("{}", &release.version));
        }

        if let Some(bx) = &utils::render_markup_widget(release.description.clone()) {
            description_box.append(bx);
        }
    }
}
