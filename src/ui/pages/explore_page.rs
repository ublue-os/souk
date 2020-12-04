use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::db::{queries, DisplayLevel};
use crate::ui::SoukPackageTile;
use crate::ui::View;

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

        get_widget!(self.builder, gtk::FlowBox, recently_updated_flowbox);
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
