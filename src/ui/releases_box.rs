use appstream_rs::Release;
use gtk::prelude::*;

use crate::ui::utils;

pub struct ReleasesBox {
    pub widget: gtk::Box,
    releases: Option<Vec<Release>>,

    builder: gtk::Builder,
}

impl ReleasesBox {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/releases_box.ui");
        get_widget!(builder, gtk::Box, releases_box);

        let releases_box = Self {
            widget: releases_box,
            releases: None,
            builder,
        };

        releases_box.setup_signals();
        releases_box
    }

    fn setup_signals(&self) {}

    pub fn set_releases(&mut self, releases: Vec<Release>) {
        if !releases.is_empty(){
            self.widget.set_visible(true);
            let release = releases[0].clone();

            get_widget!(self.builder, gtk::Label, date_label);
            get_widget!(self.builder, gtk::Label, header_label);

            utils::set_date_label(&date_label, release.date.clone());
            header_label.set_text(&format!("New in Version {}", &release.version));
        }else{
            self.widget.set_visible(false);
        }

        self.releases = Some(releases);
    }
}
