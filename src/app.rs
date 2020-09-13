use gio::subclass::prelude::ApplicationImpl;
use gio::{self, prelude::*, ApplicationFlags};
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk::subclass::application::GtkApplicationImpl;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use crate::backend::FlatpakBackend;
use crate::backend::Package;
use crate::config;
use crate::ui::pages::{ExplorePage, InstalledPage, PackageDetailsPage, SearchPage};
use crate::ui::{GsApplicationWindow, View};

#[derive(Debug, Clone)]
pub enum Action {
    ViewSet(View),
    ViewGoBack,
}

pub struct GsApplicationPrivate {
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    flatpak_backend: Rc<FlatpakBackend>,

    pub explore_page: Rc<ExplorePage>,
    pub installed_page: Rc<InstalledPage>,
    pub search_page: Rc<SearchPage>,
    pub package_details_page: Rc<PackageDetailsPage>,

    window: RefCell<Option<GsApplicationWindow>>,
}

impl ObjectSubclass for GsApplicationPrivate {
    const NAME: &'static str = "GsApplication";
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let flatpak_backend = FlatpakBackend::new();

        let explore_page = ExplorePage::new(sender.clone());
        let search_page = SearchPage::new(sender.clone());
        let installed_page = InstalledPage::new(sender.clone(), flatpak_backend.clone());
        let package_details_page = PackageDetailsPage::new(sender.clone(), flatpak_backend.clone());

        let window = RefCell::new(None);

        Self {
            sender,
            receiver,
            flatpak_backend,
            explore_page,
            installed_page,
            search_page,
            package_details_page,
            window,
        }
    }
}

// Implement GLib.OBject for GsApplication
impl ObjectImpl for GsApplicationPrivate {
    glib_object_impl!();
}

// Implement Gtk.Application for GsApplication
impl GtkApplicationImpl for GsApplicationPrivate {}

// Implement Gio.Application for GsApplication
impl ApplicationImpl for GsApplicationPrivate {
    fn activate(&self, _app: &gio::Application) {
        debug!("gio::Application -> activate()");

        // If the window already exists,
        // present it instead creating a new one again.
        if let Some(ref window) = *self.window.borrow() {
            window.present();
            info!("Application window presented.");
            return;
        }

        // No window available -> we have to create one
        let app = ObjectSubclass::get_instance(self)
            .downcast::<GsApplication>()
            .unwrap();
        let window = app.create_window();
        window.present();
        self.window.replace(Some(window));
        info!("Created application window.");

        // Setup action channel
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.process_action(action));
    }
}

// Wrap GsApplicationPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct GsApplication(
        Object<subclass::simple::InstanceStruct<GsApplicationPrivate>,
        subclass::simple::ClassStruct<GsApplicationPrivate>,
        GsApplicationClass>)
        @extends gio::Application, gtk::Application;

    match fn {
        get_type => || GsApplicationPrivate::get_type().to_glib(),
    }
}

// GsApplication implementation itself
impl GsApplication {
    pub fn run() {
        info!(
            "{} ({}) ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG
        );
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        // Create new GObject and downcast it into GsApplication
        let app = glib::Object::new(
            GsApplication::static_type(),
            &[
                ("application-id", &Some(config::APP_ID)),
                ("flags", &ApplicationFlags::empty()),
            ],
        )
        .unwrap()
        .downcast::<GsApplication>()
        .unwrap();

        app.set_resource_base_path(Some("/org/gnome/Store"));

        // Start running gtk::Application
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&app, &args);
    }

    fn create_window(&self) -> GsApplicationWindow {
        let self_ = GsApplicationPrivate::from_instance(self);
        let window = GsApplicationWindow::new(self_.sender.clone(), self.clone());

        // Load custom styling
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/org/gnome/Store/gtk/style.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &p, 500);

        // Set initial view
        window.set_view(View::Explore, false);

        window
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let self_ = GsApplicationPrivate::from_instance(self);

        match action {
            Action::ViewSet(view) => self_
                .window
                .borrow()
                .as_ref()
                .unwrap()
                .set_view(view, false),
            Action::ViewGoBack => self_.window.borrow().as_ref().unwrap().go_back(),
        }
        glib::Continue(true)
    }
}
