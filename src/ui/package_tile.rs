use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::backend::Package;
use crate::ui::utils;
use crate::ui::View;

pub struct PackageTile {
    pub widget: gtk::Button,
    package: Package,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl PackageTile {
    pub fn new(sender: Sender<Action>, package: Package) -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/package_tile.ui");
        get_widget!(builder, gtk::Button, package_tile);

        let package_tile = Self {
            widget: package_tile,
            package,
            builder,
            sender,
        };

        get_widget!(package_tile.builder, gtk::Label, title_label);
        get_widget!(package_tile.builder, gtk::Label, summary_label);

        utils::set_label_translatable_string(
            &title_label,
            Some(package_tile.package.component.name.clone()),
        );
        utils::set_label_translatable_string(
            &summary_label,
            package_tile.package.component.summary.clone(),
        );

        get_widget!(package_tile.builder, gtk::Image, icon_image);
        utils::set_icon(&package_tile.package, &icon_image, 64);

        package_tile.setup_signals();
        package_tile
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, package_tile);
        package_tile.connect_clicked(
            clone!(@strong self.sender as sender, @strong self.package as package => move |_|{
                send!(sender, Action::ViewSet(View::PackageDetails(Box::new(package.clone()))));
            }),
        );
    }
}
