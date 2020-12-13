use appstream::Release;
use gtk::prelude::*;

use crate::ui::release_row::ReleaseRow;

#[derive(Debug)]
pub struct ReleasesWindow {
    pub widget: libhandy::Window,
    builder: gtk::Builder,
    releases: Vec<Release>,
}

impl ReleasesWindow {
    pub fn new(releases: Vec<Release>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/releases_window.ui");
        get_widget!(builder, libhandy::Window, releases_window);

        let releases_window = Self {
            widget: releases_window,
            builder,
            releases,
        };
        releases_window.setup_widgets();
        releases_window
    }

    fn setup_widgets(&self) {
        let app: gtk::Application = gio::Application::get_default().unwrap().downcast().unwrap();
        let window = app.get_active_window().unwrap();
        self.widget.set_transient_for(Some(&window));
        self.widget.set_modal(true);
        get_widget!(self.builder, gtk::ListBox, listbox);
        for release in self.releases.clone().into_iter() {
            let r = ReleaseRow::new(release, false);
            listbox.append(&r.widget);
        }
    }
}
