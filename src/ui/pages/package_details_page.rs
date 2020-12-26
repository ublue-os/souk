use glib::Sender;
use gtk::prelude::*;

use std::collections::HashSet;
use std::rc::Rc;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::db::{queries, DisplayLevel};
use crate::ui::package_widgets::{
    PackageWidget, ProjectUrlsBox, ReleasesBox, ScreenshotsBox, SoukActionButton,
};
use crate::ui::{utils, SoukPackageTile, View};

pub struct PackageDetailsPage {
    pub widget: gtk::Box,
    package_widgets: Vec<Box<dyn PackageWidget>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl PackageDetailsPage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_details_page.ui");
        get_widget!(builder, gtk::Box, package_details_page);

        let mut package_widgets: Vec<Box<dyn PackageWidget>> = Vec::new();

        // ActionButton
        let wide_pab = SoukActionButton::new();
        get_widget!(builder, gtk::Box, wide_pab_box);
        wide_pab_box.append(&wide_pab);
        package_widgets.push(Box::new(wide_pab));

        let narrow_pab = SoukActionButton::new();
        get_widget!(builder, gtk::Box, narrow_pab_box);
        narrow_pab_box.append(&narrow_pab);
        package_widgets.push(Box::new(narrow_pab));

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
            let tile = row.clone().downcast::<SoukPackageTile>().unwrap();
            let package: SoukPackage = tile.get_package().unwrap();
            send!(this.sender, Action::ViewSet(View::PackageDetails(package)));
        });

        other_apps_flowbox.connect_child_activated(closure.clone());
    }

    pub fn set_package(&self, package: SoukPackage) {
        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, wide_title_label);
        get_widget!(self.builder, gtk::Label, wide_developer_label);
        get_widget!(self.builder, gtk::Label, narrow_title_label);
        get_widget!(self.builder, gtk::Label, narrow_developer_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        get_widget!(self.builder, gtk::Box, description_box);
        get_widget!(self.builder, gtk::ScrolledWindow, scrolled_window);

        // scroll up when a new package gets set
        if let Some(adj) = scrolled_window.get_vadjustment() {
            adj.set_value(0.0)
        }

        // Set icon
        utils::set_icon(&package, &icon_image, 128);

        let def = gtk::Label::new(Some(&format!("Branch: {}", package.get_branch())));

        let appdata = match package.get_appdata() {
            Some(appdata) => appdata,
            None => {
                warn!("No appdata available for package {}", package.get_name());

                // Fallback to basic information
                wide_title_label.set_text(&package.get_name());
                wide_developer_label.set_text("Unknown Developer");
                narrow_title_label.set_text(&package.get_name());
                narrow_developer_label.set_text("Unknown Developer");
                summary_label.set_text(&format!("{:?} component", package.get_kind()));
                description_box.append(&def);

                return;
            }
        };

        // Set general information
        utils::set_label_translatable_string(&wide_title_label, Some(appdata.name.clone()));
        utils::set_label_translatable_string(&wide_developer_label, appdata.developer_name.clone());
        utils::set_label_translatable_string(&narrow_title_label, Some(appdata.name.clone()));
        utils::set_label_translatable_string(
            &narrow_developer_label,
            appdata.developer_name.clone(),
        );
        utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
        if let Some(bx) = &utils::render_markup_widget(appdata.description.clone()) {
            description_box.append(bx);
        } else if package.get_appdata().is_some() {
            description_box.append(&def);
        }

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
                    tile.set_package(&p);
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

        get_widget!(self.builder, gtk::Box, description_box);
        while let Some(w) = description_box.get_first_child() {
            description_box.remove(&w);
        }

        for package_widget in &self.package_widgets {
            package_widget.reset();
        }
    }
}
