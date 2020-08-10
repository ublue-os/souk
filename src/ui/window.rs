use appstream_rs::AppId;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::{BinImpl, ContainerImpl, WidgetImpl, WindowImpl};
use libhandy::prelude::*;

use crate::app::{Action, FfApplication, FfApplicationPrivate};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Explore,
    Installed,
    Updates,
    AppDetails,
}

pub struct FfApplicationWindowPrivate {
    window_builder: gtk::Builder,
}

impl ObjectSubclass for FfApplicationWindowPrivate {
    const NAME: &'static str = "FfApplicationWindow";
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let window_builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/window.ui");

        Self { window_builder }
    }
}

// Implement GLib.OBject for FfApplicationWindow
impl ObjectImpl for FfApplicationWindowPrivate {
    glib_object_impl!();
}

// Implement Gtk.Widget for FfApplicationWindow
impl WidgetImpl for FfApplicationWindowPrivate {}

// Implement Gtk.Container for FfApplicationWindow
impl ContainerImpl for FfApplicationWindowPrivate {}

// Implement Gtk.Bin for FfApplicationWindow
impl BinImpl for FfApplicationWindowPrivate {}

// Implement Gtk.Window for FfApplicationWindow
impl WindowImpl for FfApplicationWindowPrivate {}

// Implement Gtk.ApplicationWindow for FfApplicationWindow
impl gtk::subclass::prelude::ApplicationWindowImpl for FfApplicationWindowPrivate {}

// Implement Hdy.ApplicationWindow for FfApplicationWindow
impl libhandy::subclass::prelude::ApplicationWindowImpl for FfApplicationWindowPrivate {}

// Wrap FfApplicationWindowPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct FfApplicationWindow(
        Object<subclass::simple::InstanceStruct<FfApplicationWindowPrivate>,
        subclass::simple::ClassStruct<FfApplicationWindowPrivate>,
        FfApplicationWindowClass>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow;

    match fn {
        get_type => || FfApplicationWindowPrivate::get_type().to_glib(),
    }
}

// FfApplicationWindow implementation itself
impl FfApplicationWindow {
    pub fn new(sender: Sender<Action>, app: FfApplication) -> Self {
        // Create new GObject and downcast it into FfApplicationWindow
        let window = glib::Object::new(FfApplicationWindow::static_type(), &[])
            .unwrap()
            .downcast::<FfApplicationWindow>()
            .unwrap();

        app.add_window(&window.clone());
        window.setup_widgets();
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);
        window
    }

    pub fn setup_widgets(&self) {
        let self_ = FfApplicationWindowPrivate::from_instance(self);
        let app: FfApplication = self
            .get_application()
            .unwrap()
            .downcast::<FfApplication>()
            .unwrap();
        let app_private = FfApplicationPrivate::from_instance(&app);

        // wire everything up
        get_widget!(self_.window_builder, gtk::Box, app_details_box);
        app_details_box.add(&app_private.app_details.widget);

        // Add headerbar/content to the window itself
        get_widget!(self_.window_builder, gtk::Box, window);
        self.add(&window);
    }

    fn setup_signals(&self, sender: Sender<Action>) {
        let self_ = FfApplicationWindowPrivate::from_instance(self);

        // deck
        get_widget!(self_.window_builder, libhandy::Deck, window_deck);
        window_deck.connect_property_visible_child_notify(
            clone!(@strong self as this => move |_| {
                this.sync_ui_state();
            }),
        );

        // firefox button
        get_widget!(self_.window_builder, gtk::Button, firefox_button);
        firefox_button.connect_clicked(clone!(@strong sender => move |_| {
            send!(sender, Action::ViewSet(View::AppDetails));
            send!(sender, Action::ViewShowAppDetails(AppId("org.mozilla.firefox".to_string())));
        }));
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from FfApplicationWindow to libhandy::ApplicationWindow, because FfApplicationWindow
        // currently doesn't implement GLib.ActionMap, since it's not supported in gtk-rs for subclassing (13-01-2020)
        let window = self.clone().upcast::<gtk::ApplicationWindow>();
        let app = window.get_application().unwrap();

        // win.quit
        action!(
            window,
            "quit",
            clone!(@weak app => move |_, _| {
                app.quit();
            })
        );
        app.set_accels_for_action("win.quit", &["<primary>q"]);

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

    pub fn set_view(&self, view: View) {
        self.update_view(view);
        self.sync_ui_state();
    }

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let self_ = FfApplicationWindowPrivate::from_instance(self);

        get_widget!(self_.window_builder, libhandy::Deck, window_deck);
        window_deck.navigate(libhandy::NavigationDirection::Back);

        // Make sure that the rest of the UI is correctly synced
        self.sync_ui_state();
    }

    fn sync_ui_state(&self) {
        let self_ = FfApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, libhandy::Deck, window_deck);
        get_widget!(self_.window_builder, gtk::Stack, main_stack);

        let deck_child_name = window_deck.get_visible_child_name().unwrap();
        let current_view = if deck_child_name == "app-details" {
            View::AppDetails
        } else {
            match main_stack.get_visible_child_name().unwrap().as_str() {
                "explore" => View::Explore,
                "installed" => View::Installed,
                "updates" => View::Updates,
                _ => View::Explore,
            }
        };

        debug!("Setting current view as {:?}", &current_view);
        self.update_view(current_view);
    }

    fn update_view(&self, view: View) {
        debug!("Set view to {:?}", &view);

        let self_ = FfApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, libhandy::Deck, window_deck);
        get_widget!(self_.window_builder, gtk::Stack, main_stack);

        // Show requested view / page
        match view {
            View::Explore => {
                main_stack.set_visible_child_name("explore");
                window_deck.set_visible_child_name("main");
            }
            View::Installed => {
                main_stack.set_visible_child_name("installed");
                window_deck.set_visible_child_name("main");
            }
            View::Updates => {
                main_stack.set_visible_child_name("updates");
                window_deck.set_visible_child_name("main");
            }
            View::AppDetails => {
                window_deck.set_visible_child_name("app-details");
            }
        }
    }
}
