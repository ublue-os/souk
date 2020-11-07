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

use crate::backend::SoukFlatpakBackend;
use crate::config;
use crate::ui::about_dialog;
use crate::ui::pages::{ExplorePage, InstalledPage, PackageDetailsPage, SearchPage};
use crate::ui::{SoukApplicationWindow, View};

#[derive(Debug, Clone)]
pub enum Action {
    ViewSet(View),
    ViewGoBack,
}

pub struct SoukApplicationPrivate {
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    flatpak_backend: SoukFlatpakBackend,

    pub explore_page: RefCell<Option<Rc<ExplorePage>>>,
    pub installed_page: RefCell<Option<Rc<InstalledPage>>>,
    pub search_page: RefCell<Option<Rc<SearchPage>>>,
    pub package_details_page: RefCell<Option<Rc<PackageDetailsPage>>>,

    window: RefCell<Option<SoukApplicationWindow>>,
}

impl ObjectSubclass for SoukApplicationPrivate {
    const NAME: &'static str = "SoukApplication";
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let flatpak_backend = SoukFlatpakBackend::new();

        let explore_page = RefCell::new(None);
        let search_page = RefCell::new(None);
        let installed_page = RefCell::new(None);
        let package_details_page = RefCell::new(None);

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

// Implement GLib.OBject for SoukApplication
impl ObjectImpl for SoukApplicationPrivate {}

// Implement Gtk.Application for SoukApplication
impl GtkApplicationImpl for SoukApplicationPrivate {}

// Implement Gio.Application for SoukApplication
impl ApplicationImpl for SoukApplicationPrivate {
    fn activate(&self, _app: &gio::Application) {
        debug!("Activate GIO Application...");

        // If the window already exists,
        // present it instead creating a new one again.
        if let Some(ref window) = *self.window.borrow() {
            window.present();
            info!("Application window presented.");
            return;
        }

        // No window available -> we have to create one
        let app = ObjectSubclass::get_instance(self)
            .downcast::<SoukApplication>()
            .unwrap();

        debug!("Setup Souk base components...");
        app.setup();

        debug!("Create new application window...");
        let window = app.create_window();
        window.present();
        self.window.replace(Some(window));
        info!("Created application window.");

        // Setup action channel
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.process_action(action));
    }
}

// Wrap SoukApplicationPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct SoukApplication(
        Object<subclass::simple::InstanceStruct<SoukApplicationPrivate>,
        subclass::simple::ClassStruct<SoukApplicationPrivate>,
        SoukApplicationClass>)
        @extends gio::Application, gtk::Application;

    match fn {
        get_type => || SoukApplicationPrivate::get_type().to_glib(),
    }
}

// SoukApplication implementation itself
impl SoukApplication {
    pub fn run() {
        info!(
            "{} ({}) ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG
        );
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        // Create new GObject and downcast it into SoukApplication
        let app = glib::Object::new(
            SoukApplication::static_type(),
            &[
                ("application-id", &Some(config::APP_ID)),
                ("flags", &ApplicationFlags::empty()),
            ],
        )
        .unwrap()
        .downcast::<SoukApplication>()
        .unwrap();

        app.set_default();
        app.set_resource_base_path(Some("/de/haeckerfelix/Souk"));

        // Start running gtk::Application
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&app, &args);
    }

    fn setup(&self) {
        let self_ = SoukApplicationPrivate::from_instance(self);
        let sender = self_.sender.clone();
        let flatpak_backend = self_.flatpak_backend.clone();

        flatpak_backend.init();

        *self_.explore_page.borrow_mut() = Some(ExplorePage::new(sender.clone()));
        *self_.search_page.borrow_mut() = Some(SearchPage::new(sender.clone()));
        *self_.installed_page.borrow_mut() =
            Some(InstalledPage::new(sender.clone(), flatpak_backend.clone()));
        *self_.package_details_page.borrow_mut() = Some(PackageDetailsPage::new(
            sender.clone(),
            flatpak_backend.clone(),
        ));
    }

    pub fn get_flatpak_backend(&self) -> SoukFlatpakBackend {
        let self_ = SoukApplicationPrivate::from_instance(self);
        self_.flatpak_backend.clone()
    }

    fn create_window(&self) -> SoukApplicationWindow {
        let self_ = SoukApplicationPrivate::from_instance(self);
        let window = SoukApplicationWindow::new(self_.sender.clone(), self.clone());

        // Load custom styling
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/Souk/gtk/style.css");
        gtk::StyleContext::add_provider_for_display(&gdk::Display::get_default().unwrap(), &p, 500);

        // Set initial view
        window.set_view(View::Explore, false);

        // Setup GActions
        self.setup_gactions();

        window
    }

    fn setup_gactions(&self) {
        let self_ = SoukApplicationPrivate::from_instance(self);
        let app = self.clone().upcast::<gtk::Application>();
        let window: gtk::ApplicationWindow = self.get_active_window().unwrap().downcast().unwrap();
        let sender = self_.sender.clone();

        // app.quit
        action!(
            app,
            "quit",
            clone!(@weak app => move |_, _| {
                app.quit();
            })
        );
        app.set_accels_for_action("app.quit", &["<primary>q"]);

        // app.about
        action!(
            app,
            "about",
            clone!(@weak window => move |_, _| {
                about_dialog::show_about_dialog(window);
            })
        );

        // app.search
        action!(
            app,
            "search",
            clone!(@weak window, @strong sender => move |_, _| {
                send!(sender, Action::ViewSet(View::Search));
            })
        );
        app.set_accels_for_action("app.search", &["<primary>f"]);

        // win.go-back
        action!(
            window,
            "go-back",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewGoBack);
            })
        );
        app.set_accels_for_action("win.go-back", &["Escape"]);
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let self_ = SoukApplicationPrivate::from_instance(self);

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
