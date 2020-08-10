use appstream_rs::Component;
use glib::Sender;
use gtk::prelude::*;

use crate::ui::utils;
use crate::app::Action;

pub struct AppTile {
    pub widget: gtk::Button,
    component: Component,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppTile {
    pub fn new(sender: Sender<Action>, component: Component, remote: &flatpak::Remote) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_tile.ui");
        get_widget!(builder, gtk::Button, app_tile);

        let app_tile = Self {
            widget: app_tile,
            component,
            builder,
            sender,
        };

        get_widget!(app_tile.builder, gtk::Label, title_label);
        get_widget!(app_tile.builder, gtk::Label, summary_label);

        utils::set_label_translatable_string(&title_label, Some(app_tile.component.name.clone()));
        utils::set_label_translatable_string(&summary_label, app_tile.component.summary.clone());

        get_widget!(app_tile.builder, gtk::Image, icon_image);
        utils::set_icon(&remote, &icon_image, &app_tile.component.icons[0], 64);

        app_tile.setup_signals();
        app_tile
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, app_tile);
        app_tile.connect_clicked(clone!(@strong self.sender as sender, @strong self.component as component => move |_|{
            send!(sender, Action::ViewShowAppDetails(component.id.clone()));
        }));
    }
}
