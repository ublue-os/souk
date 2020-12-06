use gio::subclass::prelude::ApplicationImpl;
use gio::{self, prelude::*, ApplicationFlags};
use glib::subclass;
use glib::subclass::prelude::*;
use glib::WeakRef;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk::subclass::application::GtkApplicationImpl;
use once_cell::unsync::OnceCell;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use crate::backend::SoukFlatpakBackend;
use crate::config;
use crate::db::SoukDatabase;
use crate::ui::about_dialog;
use crate::ui::pages::{ExplorePage, InstalledPage, LoadingPage, PackageDetailsPage, SearchPage};
use crate::ui::{SoukApplicationWindow, View};

#[derive(Debug, Clone)]
pub enum Action {
    ViewSet(View),
    ViewGoBack,

    DatabaseRebuild,
    DatabaseLoadData,
}

pub struct SoukApplicationPrivate {
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    flatpak_backend: SoukFlatpakBackend,
    database: SoukDatabase,

    pub loading_page: OnceCell<Rc<LoadingPage>>,
    pub explore_page: OnceCell<Rc<ExplorePage>>,
    pub installed_page: OnceCell<Rc<InstalledPage>>,
    pub search_page: OnceCell<Rc<SearchPage>>,
    pub package_details_page: OnceCell<Rc<PackageDetailsPage>>,

    window: OnceCell<WeakRef<SoukApplicationWindow>>,
}

impl ObjectSubclass for SoukApplicationPrivate {
    const NAME: &'static str = "SoukApplication";
    type Type = SoukApplication;
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let flatpak_backend = SoukFlatpakBackend::new();
        let database = SoukDatabase::new();

        let loading_page = OnceCell::new();
        let explore_page = OnceCell::new();
        let search_page = OnceCell::new();
        let installed_page = OnceCell::new();
        let package_details_page = OnceCell::new();

        let window = OnceCell::new();

        Self {
            sender,
            receiver,
            flatpak_backend,
            database,
            loading_page,
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
    fn activate(&self, _app: &SoukApplication) {
        debug!("Activate GIO Application...");

        // If the window already exists,
        // present it instead creating a new one again.
        if let Some(weak_win) = self.window.get() {
            let window = weak_win.upgrade().unwrap();
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
        debug!("Souk base components are ready.");

        debug!("Create new application window...");
        let window = app.create_window();
        window.present();
        self.window.set(window.downgrade()).unwrap();
        info!("Created application window.");

        // Setup action channel
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.process_action(action));
    }
}

// Wrap SoukApplicationPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct SoukApplication(ObjectSubclass<SoukApplicationPrivate>)
    @extends gio::Application, gtk::Application;
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

        let loading_page = LoadingPage::new(sender.clone(), self_.database.clone());
        let _ = self_.loading_page.set(loading_page);

        let explore_page = ExplorePage::new(sender.clone());
        let _ = self_.explore_page.set(explore_page);

        let search_page = SearchPage::new(sender.clone());
        let _ = self_.search_page.set(search_page);

        let installed_page = InstalledPage::new(sender.clone(), self_.flatpak_backend.clone());
        let _ = self_.installed_page.set(installed_page);

        let package_details_page = PackageDetailsPage::new(sender.clone());
        let _ = self_.package_details_page.set(package_details_page);

        // Setup signals
        self.setup_signals();

        // Setup database
        self_.database.init();
    }

    pub fn get_flatpak_backend(&self) -> SoukFlatpakBackend {
        let self_ = SoukApplicationPrivate::from_instance(self);
        self_.flatpak_backend.clone()
    }

    fn create_window(&self) -> SoukApplicationWindow {
        let window = SoukApplicationWindow::new(self.clone());

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

    fn setup_signals(&self) {
        let self_ = SoukApplicationPrivate::from_instance(self);

        self_.database.connect_local("notify::is-busy", false, clone!(@weak self_.database as db, @strong self_.sender as sender => @default-return None::<glib::Value>, move |_|{
            if !db.get_is_busy(){
                send!(sender, Action::DatabaseLoadData);
            }
            None
        })).unwrap();
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

        // app.rebuild-database
        action!(
            app,
            "rebuild-database",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::DatabaseRebuild);
            })
        );
        app.set_accels_for_action("app.rebuild-database", &["<primary>r"]);

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

    fn get_main_window(&self) -> SoukApplicationWindow {
        let self_ = SoukApplicationPrivate::from_instance(self);
        self_.window.get().unwrap().clone().upgrade().unwrap()
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let self_ = SoukApplicationPrivate::from_instance(self);

        match action {
            Action::ViewSet(view) => self.get_main_window().set_view(view, false),
            Action::ViewGoBack => self.get_main_window().go_back(),
            Action::DatabaseRebuild => self_.database.rebuild(),
            Action::DatabaseLoadData => {
                self_.flatpak_backend.reload_installed_packages();
                self_.explore_page.get().unwrap().clone().load_data();
            }
        }
        glib::Continue(true)
    }
}
