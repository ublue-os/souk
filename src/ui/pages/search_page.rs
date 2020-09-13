use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::database::queries;
use crate::ui::PackageTile;

pub struct SearchPage {
    pub widget: gtk::Box,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl SearchPage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/search_page.ui");
        get_widget!(builder, gtk::Box, search_page);

        let search_page = Rc::new(Self {
            widget: search_page,
            builder,
            sender,
        });

        search_page.clone().setup_widgets();
        search_page.clone().setup_signals();
        search_page
    }

    fn setup_widgets(self: Rc<Self>) {}

    fn add_tile(self: Rc<Self>, app_id: String) {
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);
        let package = queries::get_package(app_id, "stable".to_string(), "flathub".to_string())
            .unwrap()
            .unwrap();
        let tile = PackageTile::new(self.sender.clone(), package);
        editors_picks_flowbox.add(&tile.widget);
        editors_picks_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {}
}
