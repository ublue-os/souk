use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::ui::utils;
use crate::backend::Package;

pub struct AppTile {
    pub widget: gtk::Button,
    package: Package,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppTile {
    pub fn new(sender: Sender<Action>, package: Package) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_tile.ui");
        get_widget!(builder, gtk::Button, app_tile);

        let app_tile = Self {
            widget: app_tile,
            package,
            builder,
            sender,
        };

        get_widget!(app_tile.builder, gtk::Label, title_label);
        get_widget!(app_tile.builder, gtk::Label, summary_label);

        utils::set_label_translatable_string(&title_label, Some(app_tile.package.component.name.clone()));
        utils::set_label_translatable_string(&summary_label, app_tile.package.component.summary.clone());

        get_widget!(app_tile.builder, gtk::Image, icon_image);
        //utils::set_icon(&app_tile.package.get_origin(), &icon_image, &app_tile.package.component.icons[0], 64);

        app_tile.setup_signals();
        app_tile
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, app_tile);
        app_tile.connect_clicked(clone!(@strong self.sender as sender, @strong self.package as package => move |_|{
            send!(sender, Action::ViewShowAppDetails(package.clone()));
        }));
    }
}
