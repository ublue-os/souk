use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::db::{queries, DisplayLevel};
use crate::ui::utils;
use crate::ui::SoukPackageTile;
use crate::ui::View;

static EDITOR_PICKS: [&str; 7] = [
    "de.haeckerfelix.Shortwave",
    "de.haeckerfelix.Fragments",
    "org.gnome.Podcasts",
    "org.gnome.design.IconLibrary",
    "org.gnome.design.Contrast",
    "com.jetbrains.IntelliJ-IDEA-Community",
    "com.google.AndroidStudio",
];

pub struct ExplorePage {
    pub widget: gtk::Box,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl ExplorePage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/explore_page.ui");
        get_widget!(builder, gtk::Box, explore_page);

        let explore_page = Rc::new(Self {
            widget: explore_page,
            builder,
            sender,
        });

        explore_page.clone().setup_signals();
        explore_page
    }

    pub fn load_data(self: Rc<Self>) {
        // Reset old data
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);
        get_widget!(self.builder, gtk::FlowBox, recently_updated_flowbox);
        utils::clear_flowbox(&editors_picks_flowbox);
        utils::clear_flowbox(&recently_updated_flowbox);

        // Editors pick flowbox
        for app in &EDITOR_PICKS {
            self.clone().add_tile(app.to_string());
        }

        // Recently updated flowbox
        for package in queries::get_recently_updated_packages(10, DisplayLevel::Apps).unwrap() {
            let tile = SoukPackageTile::new();
            tile.set_package(&package);
            recently_updated_flowbox.insert(&tile, -1);
        }
    }

    fn add_tile(self: Rc<Self>, app_id: String) {
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);
        if let Ok(pkg_option) =
            queries::get_package(app_id, "stable".to_string(), "flathub".to_string())
        {
            if let Some(package) = pkg_option {
                let tile = SoukPackageTile::new();
                tile.set_package(&package);
                editors_picks_flowbox.insert(&tile, -1);
            }
        }
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);
        get_widget!(self.builder, gtk::FlowBox, recently_updated_flowbox);

        let closure = clone!(@weak self as this => move|_: &gtk::FlowBox, flowbox_child: &gtk::FlowBoxChild|{
            let tile = flowbox_child.clone().downcast::<SoukPackageTile>().unwrap();
            let package: SoukPackage = tile.get_package().unwrap();
            send!(this.sender, Action::ViewSet(View::PackageDetails(package)));
        });

        editors_picks_flowbox.connect_child_activated(closure.clone());
        recently_updated_flowbox.connect_child_activated(closure.clone());
    }
}
