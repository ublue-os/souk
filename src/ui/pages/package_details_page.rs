use glib::Sender;
use gtk::prelude::*;

use std::collections::HashSet;
use std::rc::Rc;

use crate::app::Action;
use crate::backend::SoukFlatpakBackend;
use crate::backend::SoukPackage;
use crate::database::{queries, DisplayLevel};
use crate::ui::package_widgets::{
    PackageActionButton, PackageWidget, ProjectUrlsBox, ReleasesBox, ScreenshotsBox,
};
use crate::ui::{utils, SoukPackageTile, View};

pub struct PackageDetailsPage {
    pub widget: gtk::Box,
    flatpak_backend: SoukFlatpakBackend,

    package_widgets: Vec<Box<dyn PackageWidget>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl PackageDetailsPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: SoukFlatpakBackend) -> Rc<Self> {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_details_page.ui");
        get_widget!(builder, gtk::Box, package_details_page);

        let mut package_widgets: Vec<Box<dyn PackageWidget>> = Vec::new();

        // PackageActionButton
        let pab = PackageActionButton::new();
        get_widget!(builder, gtk::Box, pab_box);
        pab_box.append(&pab.widget);
        package_widgets.push(Box::new(pab));

        // Screenshots
        let pw_screenshots_box = ScreenshotsBox::new();
        get_widget!(builder, gtk::Box, screenshots_box);
        screenshots_box.append(&pw_screenshots_box.widget);
        package_widgets.push(Box::new(pw_screenshots_box));

        // Releases
        let pw_releases_box = ReleasesBox::new();
        get_widget!(builder, gtk::Box, releases_box);
        releases_box.append(&pw_releases_box.widget);
        package_widgets.push(Box::new(pw_releases_box));

        // Project Urls
        let pw_project_urls_box = ProjectUrlsBox::new();
        get_widget!(builder, gtk::Box, project_urls_box);
        project_urls_box.append(&pw_project_urls_box.widget);
        package_widgets.push(Box::new(pw_project_urls_box));

        let package_details_page = Rc::new(Self {
            widget: package_details_page,
            flatpak_backend,
            package_widgets,
            builder,
            sender,
        });

        package_details_page.clone().setup_signals();
        package_details_page
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::FlowBox, other_apps_flowbox);

        let closure = clone!(@weak self as this => move|_: &gtk::FlowBox, row: &gtk::FlowBoxChild|{
            let child = row.get_child().unwrap();
            let row = child.downcast::<SoukPackageTile>().unwrap();
            let package: SoukPackage = row.get_package().unwrap();
            send!(this.sender, Action::ViewSet(View::PackageDetails(package)));
        });

        other_apps_flowbox.connect_child_activated(closure.clone());
    }

    pub fn set_package(&self, package: SoukPackage) {
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

        // Set icon
        utils::set_icon(&package, &icon_image, 128);

        let appdata = match package.get_appdata() {
            Some(appdata) => appdata,
            None => {
                warn!("No appdata available for package {}", package.get_name());

                // Fallback to basic information
                title_label.set_text(&package.get_name());
                developer_label.set_text(&format!("Source: {}", package.get_remote()));
                summary_label.set_text(&format!("{:?} component", package.get_kind()));
                description_label.set_text(&format!("Branch: {}", package.get_branch(),));

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
            get_widget!(self.builder, gtk::Box, other_apps);
            get_widget!(self.builder, gtk::Label, other_apps_label);
            get_widget!(self.builder, gtk::FlowBox, other_apps_flowbox);

            let name = n.get_default().unwrap().to_string();
            other_apps_label.set_text(&format!("Other Apps by {}", name));

            let mut names = HashSet::new();
            let packages =
                queries::get_packages_by_developer_name(&name, 10, DisplayLevel::Apps).unwrap();

            for p in packages {
                let p_name = p.get_name();
                if p_name != package.get_name() && !names.contains(&p_name) {
                    debug!("Found package {} by {}", p_name, name);
                    names.insert(p_name);

                    let tile = SoukPackageTile::new();
                    tile.set_property("package", &p).unwrap();
                    other_apps_flowbox.insert(&tile, -1);
                }
            }

            other_apps.set_visible(names.len() != 0);
        }

        // Set package for all package widgets
        for package_widget in &self.package_widgets {
            package_widget.set_package(&package);
        }
    }

    pub fn reset(&self) {
        get_widget!(self.builder, gtk::Box, other_apps);
        get_widget!(self.builder, gtk::FlowBox, other_apps_flowbox);

        utils::clear_flowbox(&other_apps_flowbox);
        other_apps.set_visible(false);

        for package_widget in &self.package_widgets {
            package_widget.reset();
        }
    }
}
