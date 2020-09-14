use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::Package;
use crate::database::queries;
use crate::ui::utils;
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

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::SearchEntry, search_entry);
        search_entry.connect_search_changed(clone!(@weak self as this => move|entry|{
            get_widget!(this.builder, gtk::FlowBox, results_flowbox);
            utils::remove_all_items(&results_flowbox);

            let text = entry.get_text().to_string();
            let packages = queries::get_packages_by_name(text, 100).unwrap();

            for package in packages{
                this.clone().add_tile(&package)
            }
        }));
    }

    fn add_tile(self: Rc<Self>, package: &Package) {
        get_widget!(self.builder, gtk::FlowBox, results_flowbox);
        let tile = PackageTile::new(self.sender.clone(), package.clone());
        results_flowbox.add(&tile.widget);
        results_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {}
}
