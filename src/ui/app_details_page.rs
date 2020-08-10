use appstream_rs::{AppId, Component};
use flatpak::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::app::Action;
use crate::appstream_cache::AppStreamCache;
use crate::ui::{ReleasesBox, ProjectUrlsBox, utils};

pub struct AppDetailsPage {
    pub widget: gtk::Box,
    appstream_cache: Rc<AppStreamCache>,

    app_id: RefCell<Option<AppId>>,
    metadata: RefCell<Option<HashMap<flatpak::Remote, Component>>>,
    active_remote: RefCell<Option<flatpak::Remote>>,

    releases_box: RefCell<ReleasesBox>,
    project_urls_box: RefCell<ProjectUrlsBox>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppDetailsPage {
    pub fn new(sender: Sender<Action>, appstream_cache: Rc<AppStreamCache>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_details_page.ui");
        get_widget!(builder, gtk::Box, app_details_page);

        let releases_box = RefCell::new(ReleasesBox::new());
        let project_urls_box = RefCell::new(ProjectUrlsBox::new());

        let app_details_page = Rc::new(Self {
            widget: app_details_page,
            appstream_cache,
            app_id: RefCell::new(None),
            metadata: RefCell::new(None),
            active_remote: RefCell::new(None),
            releases_box,
            project_urls_box,
            builder,
            sender,
        });

        app_details_page.clone().setup_widgets();
        app_details_page.clone().setup_signals();
        app_details_page
    }

    fn setup_widgets(self: Rc<Self>){
        get_widget!(self.builder, gtk::Box, releases_box);
        releases_box.add(&self.releases_box.borrow().widget);

        get_widget!(self.builder, gtk::Box, project_urls_box);
        project_urls_box.add(&self.project_urls_box.borrow().widget);
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::ComboBoxText, source_combobox);
        source_combobox.connect_changed(clone!(@strong self as this => move |cb|{
            cb.get_active_id().map(|name|{
                for (r,_c) in this.metadata.borrow().as_ref().unwrap().iter(){
                    if &r.get_name().as_ref().unwrap() == &&name {
                        *this.active_remote.borrow_mut() = Some(r.clone());
                    }
                }

                this.display_values();
            });
        }));
    }

    pub fn show_details(&self, app_id: AppId) {
        *self.app_id.borrow_mut() = Some(app_id.clone());
        *self.metadata.borrow_mut() = Some(self.appstream_cache.get_components(app_id));

        get_widget!(self.builder, gtk::ComboBoxText, source_combobox);
        source_combobox.remove_all();
        source_combobox.set_visible(self.metadata.borrow().as_ref().unwrap().len() > 1 );

        for (remote, _c) in self.metadata.borrow().as_ref().unwrap() {
            let id = remote.get_name().unwrap().to_string();
            let title = remote.get_title().unwrap().to_string();

            source_combobox.insert(0, Some(&id), &title);
            source_combobox.set_active(Some(0));
            *self.active_remote.borrow_mut() = Some(remote.clone());
        }

        self.display_values();
    }

    fn display_values(&self) {
        let metadata = self.metadata.borrow();
        let remote = self.active_remote.borrow();
        let c = metadata.as_ref().unwrap().get(remote.as_ref().unwrap()).unwrap();

        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, title_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        //get_widget!(self.builder, gtk::Label, version_label);
        get_widget!(self.builder, gtk::Label, developer_label);
        //get_widget!(self.builder, gtk::Label, project_group_label);
        //get_widget!(self.builder, gtk::Label, license_label);

        utils::set_icon(remote.as_ref().unwrap(), &icon_image, &c.icons[0], 128);
        utils::set_label_translatable_string(&title_label, Some(c.name.clone()));
        utils::set_label_translatable_string(&summary_label, c.summary.clone());
        //utils::set_label(&version_label, Some(c.releases[0].version.clone()));
        utils::set_label_translatable_string(&developer_label, c.developer_name.clone());
        //utils::set_label(&project_group_label, c.project_group.clone());
        //utils::set_license_label(&license_label, c.project_license.clone());

        self.releases_box.borrow_mut().set_releases(c.releases.clone());
        self.project_urls_box.borrow_mut().set_project_urls(c.urls.clone());
    }
}
