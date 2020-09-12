use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::database::queries;
use crate::ui::{
    utils, PackageActionButton, PackageTile, ProjectUrlsBox, ReleasesBox, ScreenshotsBox,
};

pub struct PackageDetailsPage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,
    package: Package,

    action_button: Rc<PackageActionButton>,

    screenshots_box: ScreenshotsBox,
    releases_box: ReleasesBox,
    project_urls_box: ProjectUrlsBox,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl PackageDetailsPage {
    pub fn new(
        package: Package,
        sender: Sender<Action>,
        flatpak_backend: Rc<FlatpakBackend>,
    ) -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/package_details_page.ui");
        get_widget!(builder, gtk::Box, package_details_page);

        let action_button = PackageActionButton::new(flatpak_backend.clone(), package.clone());
        let screenshots_box = ScreenshotsBox::new(package.clone());
        let releases_box = ReleasesBox::new(package.clone());
        let project_urls_box = ProjectUrlsBox::new(package.clone());

        let package_details_page = Self {
            widget: package_details_page,
            flatpak_backend,
            package,
            action_button,
            screenshots_box,
            releases_box,
            project_urls_box,
            builder,
            sender,
        };

        package_details_page.setup_widgets();
        package_details_page.setup_signals();
        package_details_page.display_values();
        package_details_page
    }

    fn setup_widgets(&self) {
        get_widget!(self.builder, gtk::Box, package_action_button_box);
        package_action_button_box.add(&self.action_button.widget);

        get_widget!(self.builder, gtk::Box, screenshots_box);
        screenshots_box.add(&self.screenshots_box.widget);

        get_widget!(self.builder, gtk::Box, releases_box);
        releases_box.add(&self.releases_box.widget);

        get_widget!(self.builder, gtk::Box, project_urls_box);
        project_urls_box.add(&self.project_urls_box.widget);
    }

    fn setup_signals(&self) {}

    fn display_values(&self) {
        let c = self.package.component.clone();

        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, title_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        get_widget!(self.builder, gtk::Label, description_label);
        //get_widget!(self.builder, gtk::Label, version_label);
        get_widget!(self.builder, gtk::Label, developer_label);
        //get_widget!(self.builder, gtk::Label, project_group_label);
        //get_widget!(self.builder, gtk::Label, license_label);

        utils::set_icon(&self.package, &icon_image, 128);
        utils::set_label_translatable_string(&title_label, Some(c.name.clone()));
        utils::set_label_translatable_string(&summary_label, c.summary.clone());
        utils::set_label_markup_translatable_string(
            &description_label,
            self.package.component.description.clone(),
        );
        //utils::set_label(&version_label, Some(c.releases[0].version.clone()));
        utils::set_label_translatable_string(&developer_label, c.developer_name.clone());
        //utils::set_label(&project_group_label, c.project_group.clone());
        //utils::set_license_label(&license_label, c.project_license.clone());

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
    }
}
