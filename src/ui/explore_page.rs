use appstream_rs::enums::Icon;
use appstream_rs::TranslatableString;
use appstream_rs::{AppId, Collection, Component};
use flatpak::prelude::*;
use flatpak::InstallationExt;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::app::Action;
use crate::appstream_cache::AppStreamCache;
use crate::ui::{AppTile, utils};

pub struct ExplorePage {
    pub widget: gtk::Box,
    appstream_cache: Rc<AppStreamCache>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl ExplorePage {
    pub fn new(sender: Sender<Action>, appstream_cache: Rc<AppStreamCache>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/explore_page.ui");
        get_widget!(builder, gtk::Box, explore_page);

        let explore_page = Rc::new(Self {
            widget: explore_page,
            appstream_cache,
            builder,
            sender,
        });

        explore_page.clone().setup_widgets();
        explore_page.clone().setup_signals();
        explore_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::FlowBox, editors_picks_flowbox);

        let firefox_components = self.appstream_cache.get_components(AppId("org.mozilla.firefox".to_string()));
        let (r,c) = firefox_components.iter().next().unwrap();
        let firefox_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&firefox_tile.widget);

        let shortwave_components = self.appstream_cache.get_components(AppId("de.haeckerfelix.Shortwave".to_string()));
        let (r,c) = shortwave_components.iter().next().unwrap();
        let shortwave_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&shortwave_tile.widget);

        let podcast_components = self.appstream_cache.get_components(AppId("org.gnome.Podcasts".to_string()));
        let (r,c) = podcast_components.iter().next().unwrap();
        let podcasts_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&podcasts_tile.widget);

        editors_picks_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {

    }
}
