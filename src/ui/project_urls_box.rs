use appstream_rs::ProjectUrl;
use gtk::prelude::*;
use libhandy::prelude::*;

pub struct ProjectUrlsBox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
}

impl ProjectUrlsBox {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/project_urls_box.ui");
        get_widget!(builder, gtk::Box, project_urls_box);

        let project_urls_box = Self { widget: project_urls_box, builder };

        project_urls_box
    }

    pub fn set_project_urls(&mut self, urls: Vec<ProjectUrl>) {
        get_widget!(self.builder, gtk::ListBox, listbox);
        listbox.set_no_show_all(false);

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

        for url in urls {
            match url {
                ProjectUrl::Donation(url) => Self::set_row(&donation_row, url),
                ProjectUrl::Translate(url) => Self::set_row(&translate_row, url),
                ProjectUrl::Homepage(url) => Self::set_row(&homepage_row, url),
                ProjectUrl::BugTracker(url) => Self::set_row(&bugtracker_row, url),
                ProjectUrl::Help(url) => Self::set_row(&help_row, url),
                ProjectUrl::Faq(url) => Self::set_row(&faq_row, url),
                ProjectUrl::Contact(url) => Self::set_row(&contact_url, url),
                _ => (),
            }
        }
    }

    fn set_row(row: &libhandy::ActionRow, url: url::Url) {
        row.set_visible(true);
        row.set_subtitle(Some(&url.to_string()));
    }
}
