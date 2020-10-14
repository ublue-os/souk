use appstream::enums::ProjectUrl;
use gtk4::prelude::*;
use libhandy4::auto::traits::*;

use crate::backend::Package;
use crate::ui::package_widgets::PackageWidget;

pub struct ProjectUrlsBox {
    pub widget: gtk4::Box,
    builder: gtk4::Builder,
}

impl ProjectUrlsBox {
    fn set_row(row: &libhandy4::ActionRow, url: url::Url) {
        row.set_visible(true);
        row.set_activatable(true);
        row.set_subtitle(Some(&url.to_string()));

        //TODO: Port this to GTK4
        /*row.connect_activated(move |_| {
            if let Err(e) = gtk4::show_uri::<gtk4::Window>(
                None,
                &url.to_string(),
                glib::get_current_time(),
            ) {
                error!("Failed to show url: {:?}", e);
            }
        });*/
    }
}

impl PackageWidget for ProjectUrlsBox {
    fn new() -> Self {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/project_urls_box.ui");
        get_widget!(builder, gtk4::Box, project_urls_box);

        let project_urls_box = Self {
            widget: project_urls_box,
            builder,
        };

        project_urls_box.reset();
        project_urls_box
    }

    fn set_package(&self, package: &dyn Package) {
        let urls = package.appdata().expect("No appdata available").urls;

        get_widget!(self.builder, gtk4::ListBox, listbox);
        get_widget!(self.builder, libhandy4::ActionRow, donation_row);
        get_widget!(self.builder, libhandy4::ActionRow, translate_row);
        get_widget!(self.builder, libhandy4::ActionRow, homepage_row);
        get_widget!(self.builder, libhandy4::ActionRow, bugtracker_row);
        get_widget!(self.builder, libhandy4::ActionRow, help_row);
        get_widget!(self.builder, libhandy4::ActionRow, faq_row);
        get_widget!(self.builder, libhandy4::ActionRow, contact_url);

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
        get_widget!(self.builder, gtk4::ListBox, listbox);
        get_widget!(self.builder, libhandy4::ActionRow, donation_row);
        get_widget!(self.builder, libhandy4::ActionRow, translate_row);
        get_widget!(self.builder, libhandy4::ActionRow, homepage_row);
        get_widget!(self.builder, libhandy4::ActionRow, bugtracker_row);
        get_widget!(self.builder, libhandy4::ActionRow, help_row);
        get_widget!(self.builder, libhandy4::ActionRow, faq_row);
        get_widget!(self.builder, libhandy4::ActionRow, contact_url);

        donation_row.set_visible(false);
        translate_row.set_visible(false);
        homepage_row.set_visible(false);
        bugtracker_row.set_visible(false);
        help_row.set_visible(false);
        faq_row.set_visible(false);
        contact_url.set_visible(false);
    }
}
