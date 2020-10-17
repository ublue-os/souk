use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::backend::Package;
use crate::ui::utils;
use crate::ui::View;

pub struct PackageTile {
    pub widget: gtk::Button,
}

impl PackageTile {
    pub fn new(sender: Sender<Action>, package: &dyn Package) -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/package_tile.ui");
        get_widget!(builder, gtk::Button, package_tile);

        let tile = Self {
            widget: package_tile,
        };

        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, summary_label);
        get_widget!(builder, gtk::Image, icon_image);
        get_widget!(builder, gtk::Button, package_tile);

        // Icon
        utils::set_icon(package, &icon_image, 64);

        match package.appdata() {
            Some(appdata) => {
                // Title
                utils::set_label_translatable_string(&title_label, Some(appdata.name.clone()));
                // Summary
                utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
            }
            None => {
                // Fallback to basic information when no appdata available
                title_label.set_text(&package.name());
                summary_label.set_text(&format!("{} - {}", package.branch(), package.remote()));
            }
        };

        let base_package = package.base_package().clone();
        package_tile.connect_clicked(clone!(@strong sender => move |_|{
            send!(sender, Action::ViewSet(View::PackageDetails(Box::new(base_package.clone()))));
        }));

        tile
    }
}
