use gtk::prelude::*;
use appstream_rs::AppId;
use glib::Sender;

use crate::app::Action;

pub struct AppDetails{
    pub widget: gtk::Box,
    app_id: Option<AppId>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl AppDetails{
    pub fn new(sender: Sender<Action>) -> Self{
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_details.ui");
        get_widget!(builder, gtk::Box, app_details);

        let details = Self{
            widget: app_details,
            app_id: None,
            builder,
            sender
        };

        details
    }

    pub fn show_details(&self, app_id: AppId){

    }
}
