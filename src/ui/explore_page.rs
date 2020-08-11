use appstream_rs::AppId;
use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::appstream_cache::AppStreamCache;
use crate::ui::AppTile;

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

        let firefox_components = self.appstream_cache.get_components_for_app_id(AppId("org.mozilla.firefox".to_string()));
        let (r,c) = firefox_components.iter().next().unwrap();
        let firefox_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&firefox_tile.widget);

        let shortwave_components = self.appstream_cache.get_components_for_app_id(AppId("de.haeckerfelix.Shortwave".to_string()));
        let (r,c) = shortwave_components.iter().next().unwrap();
        let shortwave_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&shortwave_tile.widget);

        let podcast_components = self.appstream_cache.get_components_for_app_id(AppId("org.gnome.Podcasts".to_string()));
        let (r,c) = podcast_components.iter().next().unwrap();
        let podcasts_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&podcasts_tile.widget);

        let contrast_components = self.appstream_cache.get_components_for_app_id(AppId("org.gnome.design.Contrast".to_string()));
        let (r,c) = contrast_components.iter().next().unwrap();
        let contrast_tile = AppTile::new(self.sender.clone(), c.clone(), &r);
        editors_picks_flowbox.add(&contrast_tile.widget);

        editors_picks_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {

    }
}
