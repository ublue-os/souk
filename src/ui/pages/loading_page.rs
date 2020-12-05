use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::config;
use crate::db::SoukDatabase;
use crate::ui::View;

pub struct LoadingPage {
    pub widget: gtk::Box,
    database: SoukDatabase,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl LoadingPage {
    pub fn new(sender: Sender<Action>, database: SoukDatabase) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/loading_page.ui");
        get_widget!(builder, gtk::Box, loading_page);

        let loading_page = Rc::new(Self {
            widget: loading_page,
            database,
            builder,
            sender,
        });

        loading_page.clone().setup_widgets();
        loading_page.clone().setup_signals();
        loading_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Image, image);
        image.set_from_icon_name(Some(&config::APP_ID));
        image.set_pixel_size(192);
    }

    fn setup_signals(self: Rc<Self>) {
        self.database.connect_local("notify::is-busy", false, clone!(@strong self.sender as sender, @weak self.database as db => @default-return None::<glib::Value>, move |_|{
            if db.get_is_busy() {
                send!(sender, Action::ViewSet(View::Loading));
            }else{
                send!(sender, Action::ViewSet(View::Explore));
            }

            None
        })).unwrap();

        get_widget!(self.builder, gtk::ProgressBar, progressbar);
        self.database.connect_local("notify::percentage", false, clone!(@weak progressbar, @weak self.database as db => @default-return None::<glib::Value>, move |_|{
            progressbar.set_fraction(db.get_percentage());
            None
        })).unwrap();

        get_widget!(self.builder, gtk::Label, label);
        self.database.connect_local("notify::remote", false, clone!(@weak label, @weak self.database as db => @default-return None::<glib::Value>, move |_|{
            label.set_text(&format!("Parsing metadata from remote \"{}\"", db.get_remote()));
            None
        })).unwrap();
    }
}
