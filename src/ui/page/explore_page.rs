use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::ui::AppTile;

pub struct ExplorePage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl ExplorePage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/explore_page.ui");
        get_widget!(builder, gtk::Box, explore_page);

        let explore_page = Rc::new(Self {
            widget: explore_page,
            flatpak_backend,
            builder,
            sender,
        });

        //explore_page.clone().setup_widgets();
        explore_page.clone().setup_signals();
        explore_page
    }

    fn setup_widgets(self: Rc<Self>) {
        self.clone().add_tile("de.haeckerfelix.Shortwave".to_string());
        self.clone().add_tile("de.haeckerfelix.Fragments".to_string());
        self.clone().add_tile("org.gnome.Podcasts".to_string());
        self.clone().add_tile("org.gnome.design.IconLibrary".to_string());
        self.clone().add_tile("org.gnome.design.Contrast".to_string());
    }

    fn add_tile(self: Rc<Self>, app_id: String) {
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);
        let package = self.flatpak_backend.clone().get_package("app".to_string(), app_id, "x86_64".to_string(), "stable".to_string()).unwrap();
        let tile = AppTile::new(self.sender.clone(), package);
        editors_picks_flowbox.add(&tile.widget);
        editors_picks_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {}
}
