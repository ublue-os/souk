use glib::Sender;
use gtk4::prelude::*;

use std::collections::HashSet;
use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::database::{queries, DisplayLevel};
use crate::ui::package_widgets::{PackageWidget, ProjectUrlsBox, ReleasesBox, ScreenshotsBox};
use crate::ui::{utils, PackageActionButton, PackageTile};

pub struct PackageDetailsPage {
    pub widget: gtk4::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    package_widgets: Vec<Box<dyn PackageWidget>>,

    builder: gtk4::Builder,
    sender: Sender<Action>,
}

impl PackageDetailsPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/package_details_page.ui");
        get_widget!(builder, gtk4::Box, package_details_page);

        let mut package_widgets: Vec<Box<dyn PackageWidget>> = Vec::new();

        // Screenshots
        let pw_screenshots_box = ScreenshotsBox::new();
        get_widget!(builder, gtk4::Box, screenshots_box);
        screenshots_box.append(&pw_screenshots_box.widget);
        package_widgets.push(Box::new(pw_screenshots_box));

        // Releases
        let pw_releases_box = ReleasesBox::new();
        get_widget!(builder, gtk4::Box, releases_box);
        releases_box.append(&pw_releases_box.widget);
        package_widgets.push(Box::new(pw_releases_box));

        // Project Urls
        let pw_project_urls_box = ProjectUrlsBox::new();
        get_widget!(builder, gtk4::Box, project_urls_box);
        project_urls_box.append(&pw_project_urls_box.widget);
        package_widgets.push(Box::new(pw_project_urls_box));

        let package_details_page = Rc::new(Self {
            widget: package_details_page,
            flatpak_backend,
            package_widgets,
            builder,
            sender,
        });

        package_details_page.setup_signals();
        package_details_page
    }

    fn setup_signals(&self) {}

    pub fn set_package(&self, package: &dyn Package) {
        get_widget!(self.builder, gtk4::Image, icon_image);
        get_widget!(self.builder, gtk4::Label, title_label);
        get_widget!(self.builder, gtk4::Label, developer_label);
        get_widget!(self.builder, gtk4::Label, summary_label);
        get_widget!(self.builder, gtk4::Label, description_label);
        get_widget!(self.builder, gtk4::ScrolledWindow, scrolled_window);

        // scroll up when a new package gets set
        if let Some(adj) = scrolled_window.get_vadjustment() {
            adj.set_value(0.0)
        }

        // Setup package action button
        get_widget!(self.builder, gtk4::Box, package_action_button_box);
        let action_button = PackageActionButton::new(self.flatpak_backend.clone(), package);
        package_action_button_box.append(&action_button.widget);

        // Set icon
        utils::set_icon(package, &icon_image, 128);

        let appdata = match package.appdata() {
            Some(appdata) => appdata,
            None => {
                warn!("No appdata available for package {}", package.name());

                // Fallback to basic information
                title_label.set_text(&package.name());
                developer_label.set_text(&format!("Source: {}", package.remote()));
                summary_label.set_text(&format!("System {} component", package.kind().to_string()));
                description_label.set_text(&format!(
                    "Branch: {}\nCommit: {}",
                    package.branch(),
                    package.commit()
                ));

                return;
            }
        };

        // Set general information
        utils::set_label_translatable_string(&title_label, Some(appdata.name.clone()));
        utils::set_label_translatable_string(&developer_label, appdata.developer_name.clone());
        utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
        utils::set_label_markup_translatable_string(
            &description_label,
            appdata.description.clone(),
        );

        // Populate "Other Apps by X" flowbox
        if let Some(n) = appdata.developer_name {
            get_widget!(self.builder, gtk4::Box, other_apps);
            get_widget!(self.builder, gtk4::Label, other_apps_label);
            get_widget!(self.builder, gtk4::FlowBox, other_apps_flowbox);

            let name = n.get_default().unwrap().to_string();
            other_apps_label.set_text(&format!("Other Apps by {}", name));

            let mut names = HashSet::new();
            for pkg in
                queries::get_packages_by_developer_name(&name, 10, DisplayLevel::Apps).unwrap()
            {
                let pkg_name = pkg.name();
                if pkg_name != package.name() && !names.contains(&pkg_name) {
                    debug!("Found package {} by {}", pkg_name, name);
                    names.insert(pkg_name);
                    let tile = PackageTile::new(self.sender.clone(), &pkg);
                    other_apps_flowbox.insert(&tile.widget, -1);
                }
            }

            other_apps.set_visible(names.len() != 0);
        }

        // Set package for all package widgets
        for package_widget in &self.package_widgets {
            package_widget.set_package(package);
        }
    }

    pub fn reset(&self) {
        get_widget!(self.builder, gtk4::Box, other_apps);
        get_widget!(self.builder, gtk4::FlowBox, other_apps_flowbox);
        get_widget!(self.builder, gtk4::Box, package_action_button_box);

        utils::remove_all_items(&package_action_button_box);
        other_apps.set_visible(false);
        utils::remove_all_items(&other_apps_flowbox);

        for package_widget in &self.package_widgets {
            package_widget.reset();
        }
    }
}
