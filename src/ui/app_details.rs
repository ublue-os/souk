use appstream_rs::enums::Icon;
use appstream_rs::TranslatableString;
use appstream_rs::{AppId, Collection, Component};
use flatpak::prelude::*;
use flatpak::InstallationExt;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::utils;
use crate::app::Action;
use crate::appstream_cache::AppStreamCache;

pub struct AppDetails {
    pub widget: gtk::Box,
    appstream_cache: Rc<AppStreamCache>,

    app_id: RefCell<Option<AppId>>,
    metadata: RefCell<Option<HashMap<flatpak::Remote, Component>>>,
    active_remote: RefCell<Option<flatpak::Remote>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppDetails {
    pub fn new(sender: Sender<Action>, appstream_cache: Rc<AppStreamCache>) -> Rc<Self> {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_details.ui");
        get_widget!(builder, gtk::Box, app_details);

        let details = Rc::new(Self {
            widget: app_details,
            appstream_cache,
            app_id: RefCell::new(None),
            metadata: RefCell::new(None),
            active_remote: RefCell::new(None),
            builder,
            sender,
        });

        details.clone().setup_signals();
        details
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
        source_combobox.set_visible(false);

        if self.metadata.borrow().as_ref().unwrap().len() > 1 {
            source_combobox.set_visible(true);

            for (remote, _c) in self.metadata.borrow().as_ref().unwrap() {
                let id = remote.get_name().unwrap().to_string();
                let title = remote.get_title().unwrap().to_string();

                source_combobox.insert(0, Some(&id), &title);
                source_combobox.set_active(Some(0));
                *self.active_remote.borrow_mut() = Some(remote.clone());
            }
        }

        self.display_values();
    }

    fn display_values(&self) {
        let metadata = self.metadata.borrow();
        let remote = self.active_remote.borrow();
        let c = metadata
            .as_ref()
            .unwrap()
            .get(remote.as_ref().unwrap())
            .unwrap();

        get_widget!(self.builder, gtk::Image, icon_image);
        get_widget!(self.builder, gtk::Label, title_label);
        get_widget!(self.builder, gtk::Label, summary_label);
        get_widget!(self.builder, gtk::Label, version_label);
        get_widget!(self.builder, gtk::Label, developer_label);
        get_widget!(self.builder, gtk::Label, project_group_label);
        get_widget!(self.builder, gtk::Label, license_label);

        utils::set_icon(remote.as_ref().unwrap(), icon_image, &c.icons[0]);
        utils::set_label_translatable_string(title_label, Some(c.name.clone()));
        utils::set_label_translatable_string(summary_label, c.summary.clone());
        utils::set_label(version_label, Some(c.releases[0].version.clone()));
        utils::set_label_translatable_string(developer_label, c.developer_name.clone());
        utils::set_label(project_group_label, c.project_group.clone());
        utils::set_license_label(license_label, c.project_license.clone());
    }
}
