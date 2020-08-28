use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::Action;
use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::ui::{utils, AppButtonsBox, ProjectUrlsBox, ReleasesBox, ScreenshotsBox};

pub struct AppDetailsPage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,
    package: RefCell<Option<Package>>,

    app_buttons_box: RefCell<AppButtonsBox>,
    screenshots_box: RefCell<ScreenshotsBox>,
    releases_box: RefCell<ReleasesBox>,
    project_urls_box: RefCell<ProjectUrlsBox>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppDetailsPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_details_page.ui");
        get_widget!(builder, gtk::Box, app_details_page);

        let package = RefCell::new(None);

        let app_buttons_box = RefCell::new(AppButtonsBox::new(flatpak_backend.clone()));
        let screenshots_box = RefCell::new(ScreenshotsBox::new());
        let releases_box = RefCell::new(ReleasesBox::new());
        let project_urls_box = RefCell::new(ProjectUrlsBox::new());

        let app_details_page = Rc::new(Self {
            widget: app_details_page,
            flatpak_backend,
            package,
            app_buttons_box,
            screenshots_box,
            releases_box,
            project_urls_box,
            builder,
            sender,
        });

        app_details_page.clone().setup_widgets();
        app_details_page.clone().setup_signals();
        app_details_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Box, app_buttons_box);
        app_buttons_box.add(&self.app_buttons_box.borrow().widget);

        get_widget!(self.builder, gtk::Box, screenshots_box);
        screenshots_box.add(&self.screenshots_box.borrow().widget);

        get_widget!(self.builder, gtk::Box, releases_box);
        releases_box.add(&self.releases_box.borrow().widget);

        get_widget!(self.builder, gtk::Box, project_urls_box);
        project_urls_box.add(&self.project_urls_box.borrow().widget);
    }

    fn setup_signals(self: Rc<Self>) {

    }

    pub fn show_details(&self, package: Package) {
        *self.package.borrow_mut() = Some(package);

        self.display_values();
    }

    fn display_values(&self) {
        let package = self.package.borrow().clone().to_owned();
        let c = package.clone().unwrap().component.clone();

        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, title_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        //get_widget!(self.builder, gtk::Label, version_label);
        get_widget!(self.builder, gtk::Label, developer_label);
        //get_widget!(self.builder, gtk::Label, project_group_label);
        //get_widget!(self.builder, gtk::Label, license_label);

        //utils::set_icon(remote.as_ref().unwrap(), &icon_image, &c.icons[0], 128);
        utils::set_label_translatable_string(&title_label, Some(c.name.clone()));
        utils::set_label_translatable_string(&summary_label, c.summary.clone());
        //utils::set_label(&version_label, Some(c.releases[0].version.clone()));
        utils::set_label_translatable_string(&developer_label, c.developer_name.clone());
        //utils::set_label(&project_group_label, c.project_group.clone());
        //utils::set_license_label(&license_label, c.project_license.clone());

        self.app_buttons_box.borrow_mut().set_package(package.unwrap().clone());
        self.screenshots_box.borrow_mut().set_screenshots(c.screenshots.clone());
        self.releases_box.borrow_mut().set_releases(c.releases.clone());
        self.project_urls_box.borrow_mut().set_project_urls(c.urls.clone());
    }
}
