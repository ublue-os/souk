use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::database::queries;
use crate::ui::package_widgets::{PackageWidget, ProjectUrlsBox, ReleasesBox, ScreenshotsBox};
use crate::ui::{utils, PackageActionButton, PackageTile};

pub struct PackageDetailsPage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    package_widgets: Vec<Box<dyn PackageWidget>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl PackageDetailsPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/package_details_page.ui");
        get_widget!(builder, gtk::Box, package_details_page);

        let mut package_widgets: Vec<Box<dyn PackageWidget>> = Vec::new();

        // Screenshots
        let pw_screenshots_box = ScreenshotsBox::new();
        get_widget!(builder, gtk::Box, screenshots_box);
        screenshots_box.add(&pw_screenshots_box.widget);
        package_widgets.push(Box::new(pw_screenshots_box));

        // Releases
        let pw_releases_box = ReleasesBox::new();
        get_widget!(builder, gtk::Box, releases_box);
        releases_box.add(&pw_releases_box.widget);
        package_widgets.push(Box::new(pw_releases_box));

        // Project Urls
        let pw_project_urls_box = ProjectUrlsBox::new();
        get_widget!(builder, gtk::Box, project_urls_box);
        project_urls_box.add(&pw_project_urls_box.widget);
        package_widgets.push(Box::new(pw_project_urls_box));

        let package_details_page = Rc::new(Self {
            widget: package_details_page,
            flatpak_backend,
            package_widgets,
            /*action_button,
            releases_box,*/
            builder,
            sender,
        });

        package_details_page.setup_signals();
        package_details_page
    }

    fn setup_signals(&self) {}

    pub fn set_package(&self, package: Package) {
        let c = package.component.clone();

        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, title_label);
        get_widget!(self.builder, gtk::Label, developer_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        get_widget!(self.builder, gtk::Label, description_label);
        get_widget!(self.builder, gtk::ScrolledWindow, scrolled_window);

        // scroll up when a new package gets set
        if let Some(adj) = scrolled_window.get_vadjustment() {
            adj.set_value(0.0)
        }

        // Set general information
        utils::set_icon(&package, &icon_image, 128);
        utils::set_label_translatable_string(&title_label, Some(c.name.clone()));
        utils::set_label_translatable_string(&developer_label, c.developer_name.clone());
        utils::set_label_translatable_string(&summary_label, c.summary.clone());
        utils::set_label_markup_translatable_string(
            &description_label,
            package.component.description.clone(),
        );

        // Setup package action button
        get_widget!(self.builder, gtk::Box, package_action_button_box);
        let action_button = PackageActionButton::new(self.flatpak_backend.clone(), package.clone());
        package_action_button_box.add(&action_button.widget);

        // Populate "Other Apps by X" flowbox
        if let Some(n) = c.developer_name {
            get_widget!(self.builder, gtk::Box, other_apps);
            get_widget!(self.builder, gtk::Label, other_apps_label);
            get_widget!(self.builder, gtk::FlowBox, other_apps_flowbox);

            let name = n.get_default().unwrap().to_string();
            other_apps_label.set_text(&format!("Other Apps by {}", name));
            other_apps.set_visible(true);

            for package in queries::get_packages_by_developer_name(name, 10).unwrap() {
                let tile = PackageTile::new(self.sender.clone(), package);
                other_apps_flowbox.add(&tile.widget);
                other_apps_flowbox.show_all();
            }
        }

        // Set package for all package widgets
        for package_widget in &self.package_widgets {
            package_widget.set_package(package.clone());
        }
    }

    pub fn reset(&self) {
        get_widget!(self.builder, gtk::Box, other_apps);
        get_widget!(self.builder, gtk::FlowBox, other_apps_flowbox);
        get_widget!(self.builder, gtk::Box, package_action_button_box);

        utils::remove_all_items(&package_action_button_box);
        other_apps.set_visible(false);
        utils::remove_all_items(&other_apps_flowbox);

        for package_widget in &self.package_widgets {
            package_widget.reset();
        }
    }
}
