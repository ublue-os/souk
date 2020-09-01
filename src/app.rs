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
use crate::ui::page::{ExplorePage, InstalledPage, PackageDetailsPage};
use crate::ui::{FfApplicationWindow, View};

#[derive(Debug, Clone)]
pub enum Action {
    ViewSet(View),
    ViewShowAppDetails(Package),
    ViewGoBack,
}

pub struct FfApplicationPrivate {
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    flatpak_backend: Rc<FlatpakBackend>,

    pub explore_page: Rc<ExplorePage>,
    pub installed_page: Rc<InstalledPage>,

    window: RefCell<Option<FfApplicationWindow>>,
}

impl ObjectSubclass for FfApplicationPrivate {
    const NAME: &'static str = "FfApplication";
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let flatpak_backend = FlatpakBackend::new();

        let explore_page = ExplorePage::new(sender.clone(), flatpak_backend.clone());
        let installed_page = InstalledPage::new(sender.clone(), flatpak_backend.clone());

        let window = RefCell::new(None);

        Self {
            sender,
            receiver,
            flatpak_backend,
            explore_page,
            installed_page,
            window,
        }
    }
}

// Implement GLib.OBject for FfApplication
impl ObjectImpl for FfApplicationPrivate {
    glib_object_impl!();
}

// Implement Gtk.Application for FfApplication
impl GtkApplicationImpl for FfApplicationPrivate {}

// Implement Gio.Application for FfApplication
impl ApplicationImpl for FfApplicationPrivate {
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
            .downcast::<FfApplication>()
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

// Wrap FfApplicationPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct FfApplication(
        Object<subclass::simple::InstanceStruct<FfApplicationPrivate>,
        subclass::simple::ClassStruct<FfApplicationPrivate>,
        FfApplicationClass>)
        @extends gio::Application, gtk::Application;

    match fn {
        get_type => || FfApplicationPrivate::get_type().to_glib(),
    }
}

// FfApplication implementation itself
impl FfApplication {
    pub fn run() {
        info!(
            "{} ({}) ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG
        );
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        // Create new GObject and downcast it into FfApplication
        let app = glib::Object::new(
            FfApplication::static_type(),
            &[
                ("application-id", &Some(config::APP_ID)),
                ("flags", &ApplicationFlags::empty()),
            ],
        )
        .unwrap()
        .downcast::<FfApplication>()
        .unwrap();

        // Start running gtk::Application
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&app, &args);
    }

    fn create_window(&self) -> FfApplicationWindow {
        let self_ = FfApplicationPrivate::from_instance(self);
        let window = FfApplicationWindow::new(self_.sender.clone(), self.clone());

        // Load custom styling
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/FlatpakFrontend/gtk/style.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &p, 500);

        // Set initial view
        window.set_view(View::Explore);

        window
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let self_ = FfApplicationPrivate::from_instance(self);

        match action {
            Action::ViewSet(view) => self_.window.borrow().as_ref().unwrap().set_view(view),
            Action::ViewShowAppDetails(package) => {
                let page = PackageDetailsPage::new(
                    package,
                    self_.sender.clone(),
                    self_.flatpak_backend.clone(),
                );
                self_
                    .window
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .add_package_details_page(page);
            }
            Action::ViewGoBack => self_.window.borrow().as_ref().unwrap().go_back(),
        }
        glib::Continue(true)
    }
}
