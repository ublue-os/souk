use appstream_rs::AppId;
use gtk::prelude::*;
use flatpak::Remote;

use std::rc::Rc;

use crate::appstream_cache::AppStreamCache;

pub struct AppButtonsBox {
    pub widget: gtk::Box,
    appstream_cache: Rc<AppStreamCache>,

    builder: gtk::Builder,
}

impl AppButtonsBox {
    pub fn new(appstream_cache: Rc<AppStreamCache>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/app_buttons_box.ui");
        get_widget!(builder, gtk::Box, app_buttons_box);

        let app_buttons_box = Self {
            widget: app_buttons_box,
            appstream_cache,
            builder,
        };

        app_buttons_box
    }

    pub fn set_app(&mut self, app_id: AppId, remote: Remote){
        get_widget!(self.builder, gtk::Stack, button_stack);

        match self.appstream_cache.is_installed(app_id, Some(&remote)){
            true => {
                button_stack.set_visible_child_name("installed");
            },
            false => {
                button_stack.set_visible_child_name("install");
            }
        };
    }
}
