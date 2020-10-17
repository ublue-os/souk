use glib::Sender;
use gtk4::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::database::{queries, DisplayLevel};
use crate::ui::PackageTile;

pub struct ExplorePage {
    pub widget: gtk4::Box,

    builder: gtk4::Builder,
    sender: Sender<Action>,
}

impl ExplorePage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/explore_page.ui");
        get_widget!(builder, gtk4::Box, explore_page);

        let explore_page = Rc::new(Self {
            widget: explore_page,
            builder,
            sender,
        });

        explore_page.clone().setup_widgets();
        explore_page.clone().setup_signals();
        explore_page
    }

    fn setup_widgets(self: Rc<Self>) {
        self.clone()
            .add_tile("de.haeckerfelix.Shortwave".to_string());
        self.clone()
            .add_tile("de.haeckerfelix.Fragments".to_string());
        self.clone().add_tile("org.gnome.Podcasts".to_string());
        self.clone()
            .add_tile("org.gnome.design.IconLibrary".to_string());
        self.clone()
            .add_tile("org.gnome.design.Contrast".to_string());
        self.clone()
            .add_tile("com.google.AndroidStudio".to_string());
        self.clone()
            .add_tile("com.jetbrains.IntelliJ-IDEA-Community".to_string());

        get_widget!(self.builder, gtk4::FlowBox, recently_updated_flowbox);
        for package in queries::get_recently_updated_packages(10, DisplayLevel::Apps).unwrap() {
            let tile = PackageTile::new(self.sender.clone(), &package);
            recently_updated_flowbox.insert(&tile.widget, -1);
        }
    }

    fn add_tile(self: Rc<Self>, app_id: String) {
        get_widget!(self.builder, gtk4::FlowBox, editors_picks_flowbox);
        let package = queries::get_package(app_id, "stable".to_string(), "flathub".to_string())
            .unwrap()
            .unwrap();
        let tile = PackageTile::new(self.sender.clone(), &package);
        editors_picks_flowbox.insert(&tile.widget, -1);
    }

    fn setup_signals(self: Rc<Self>) {}
}
