use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::database::queries;
use crate::ui::AppTile;

pub struct ExplorePage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl ExplorePage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/explore_page.ui");
        get_widget!(builder, gtk::Box, explore_page);

        let explore_page = Rc::new(Self {
            widget: explore_page,
            flatpak_backend,
            builder,
            sender,
        });

        explore_page.clone().setup_widgets();
        explore_page.clone().setup_signals();
        explore_page
    }

    fn setup_widgets(self: Rc<Self>) {
        self.clone()
            .add_tile("de.haeckerfelix.Shortwave.Devel".to_string());
    }

    fn add_tile(self: Rc<Self>, app_id: String) {
        dbg!(&app_id);
        get_widget!(self.builder, gtk::FlowBox, recently_updated_flowbox);
        let package =
            queries::get_package(app_id, "master".to_string(), "rust_nightly".to_string())
                .unwrap()
                .unwrap();
        let tile = AppTile::new(self.sender.clone(), package);
        recently_updated_flowbox.add(&tile.widget);
        recently_updated_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {}
}
