use appstream::Release;
use gtk::prelude::*;

use crate::ui::version_row::VersionRow;

#[derive(Debug)]
pub struct VersionsWindow {
    pub widget: libhandy::Window,
    builder: gtk::Builder,
    releases: Vec<Release>,
}

impl VersionsWindow {
    pub fn new(releases: Vec<Release>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/versions_window.ui");
        get_widget!(builder, libhandy::Window, versions_window);

        let versions_window = Self {
            widget: versions_window,
            builder,
            releases,
        };
        versions_window.setup_widgets();
        versions_window
    }

    fn setup_widgets(&self) {
        let app: gtk::Application = gio::Application::get_default().unwrap().downcast().unwrap();
        let window = app.get_active_window().unwrap();
        self.widget.set_transient_for(Some(&window));
        self.widget.set_modal(true);
        get_widget!(self.builder, gtk::ListBox, listbox);
        for version in self.releases.clone().into_iter() {
            let r = VersionRow::new(version, false);
            listbox.append(&r.widget);
        }
    }
}
