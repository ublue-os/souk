use appstream::enums::ProjectUrl;
use gtk::prelude::*;
use libhandy::prelude::*;

use crate::backend::Package;
use crate::ui::package_widgets::PackageWidget;

pub struct ProjectUrlsBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
}

impl ProjectUrlsBox {
    fn set_row(row: &libhandy::ActionRow, url: url::Url) {
        row.set_visible(true);
        row.set_activatable(true);
        row.set_subtitle(Some(&url.to_string()));

        row.connect_activated(move |_| {
            if let Err(e) = gtk::show_uri_on_window::<gtk::Window>(
                None,
                &url.to_string(),
                gtk::get_current_event_time(),
            ) {
                error!("Failed to show url: {:?}", e);
            }
        });
    }
}

impl PackageWidget for ProjectUrlsBox {
    fn new() -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/project_urls_box.ui");
        get_widget!(builder, gtk::Box, project_urls_box);

        let project_urls_box = Self {
            widget: project_urls_box,
            builder,
        };

        project_urls_box.reset();
        project_urls_box
    }

    fn set_package(&self, package: &dyn Package) {
        let urls = package.appdata().expect("No appdata available").urls;

        get_widget!(self.builder, gtk::ListBox, listbox);
        get_widget!(self.builder, libhandy::ActionRow, donation_row);
        get_widget!(self.builder, libhandy::ActionRow, translate_row);
        get_widget!(self.builder, libhandy::ActionRow, homepage_row);
        get_widget!(self.builder, libhandy::ActionRow, bugtracker_row);
        get_widget!(self.builder, libhandy::ActionRow, help_row);
        get_widget!(self.builder, libhandy::ActionRow, faq_row);
        get_widget!(self.builder, libhandy::ActionRow, contact_url);

        for url in &urls {
            match url {
                ProjectUrl::Donation(url) => Self::set_row(&donation_row, url.to_owned()),
                ProjectUrl::Translate(url) => Self::set_row(&translate_row, url.to_owned()),
                ProjectUrl::Homepage(url) => Self::set_row(&homepage_row, url.to_owned()),
                ProjectUrl::BugTracker(url) => Self::set_row(&bugtracker_row, url.to_owned()),
                ProjectUrl::Help(url) => Self::set_row(&help_row, url.to_owned()),
                ProjectUrl::Faq(url) => Self::set_row(&faq_row, url.to_owned()),
                ProjectUrl::Contact(url) => Self::set_row(&contact_url, url.to_owned()),
                _ => (),
            }
        }
    }

    fn reset(&self) {
        get_widget!(self.builder, gtk::ListBox, listbox);
        get_widget!(self.builder, libhandy::ActionRow, donation_row);
        get_widget!(self.builder, libhandy::ActionRow, translate_row);
        get_widget!(self.builder, libhandy::ActionRow, homepage_row);
        get_widget!(self.builder, libhandy::ActionRow, bugtracker_row);
        get_widget!(self.builder, libhandy::ActionRow, help_row);
        get_widget!(self.builder, libhandy::ActionRow, faq_row);
        get_widget!(self.builder, libhandy::ActionRow, contact_url);

        donation_row.set_visible(false);
        translate_row.set_visible(false);
        homepage_row.set_visible(false);
        bugtracker_row.set_visible(false);
        help_row.set_visible(false);
        faq_row.set_visible(false);
        contact_url.set_visible(false);
    }
}
