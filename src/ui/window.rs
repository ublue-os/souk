use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::{BinImpl, ContainerImpl, WidgetImpl, WindowImpl};
use libhandy::prelude::*;

use std::cell::RefCell;

use crate::app::{Action, GsApplication, GsApplicationPrivate};
use crate::backend::Package;
use crate::ui::about_dialog;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Explore,
    Installed,
    Updates,
    Search,
    PackageDetails(Box<Package>),
}

pub struct GsApplicationWindowPrivate {
    window_builder: gtk::Builder,
    menu_builder: gtk::Builder,

    pages_stack: RefCell<Vec<View>>,
}

impl ObjectSubclass for GsApplicationWindowPrivate {
    const NAME: &'static str = "GsApplicationWindow";
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let window_builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/window.ui");
        let menu_builder = gtk::Builder::from_resource("/org/gnome/Store/gtk/menu.ui");

        let pages_stack = RefCell::new(Vec::new());

        Self {
            window_builder,
            menu_builder,
            pages_stack,
        }
    }
}

// Implement GLib.OBject for GsApplicationWindow
impl ObjectImpl for GsApplicationWindowPrivate {
    glib_object_impl!();
}

// Implement Gtk.Widget for GsApplicationWindow
impl WidgetImpl for GsApplicationWindowPrivate {}

// Implement Gtk.Container for GsApplicationWindow
impl ContainerImpl for GsApplicationWindowPrivate {}

// Implement Gtk.Bin for GsApplicationWindow
impl BinImpl for GsApplicationWindowPrivate {}

// Implement Gtk.Window for GsApplicationWindow
impl WindowImpl for GsApplicationWindowPrivate {}

// Implement Gtk.ApplicationWindow for GsApplicationWindow
impl gtk::subclass::prelude::ApplicationWindowImpl for GsApplicationWindowPrivate {}

// Implement Hdy.ApplicationWindow for GsApplicationWindow
impl libhandy::subclass::prelude::ApplicationWindowImpl for GsApplicationWindowPrivate {}

// Wrap GsApplicationWindowPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct GsApplicationWindow(
        Object<subclass::simple::InstanceStruct<GsApplicationWindowPrivate>,
        subclass::simple::ClassStruct<GsApplicationWindowPrivate>,
        GsApplicationWindowClass>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow;

    match fn {
        get_type => || GsApplicationWindowPrivate::get_type().to_glib(),
    }
}

// GsApplicationWindow implementation itself
impl GsApplicationWindow {
    pub fn new(sender: Sender<Action>, app: GsApplication) -> Self {
        // Create new GObject and downcast it into GsApplicationWindow
        let window = glib::Object::new(GsApplicationWindow::static_type(), &[])
            .unwrap()
            .downcast::<GsApplicationWindow>()
            .unwrap();

        app.add_window(&window);
        window.setup_widgets();
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);
        window
    }

    pub fn setup_widgets(&self) {
        let self_ = GsApplicationWindowPrivate::from_instance(self);
        let app: GsApplication = self
            .get_application()
            .unwrap()
            .downcast::<GsApplication>()
            .unwrap();
        let app_private = GsApplicationPrivate::from_instance(&app);

        // set default size
        self.set_default_size(900, 700);

        // Set hamburger menu
        get_widget!(self_.menu_builder, gtk::PopoverMenu, popover_menu);
        get_widget!(self_.window_builder, gtk::MenuButton, appmenu_button);
        appmenu_button.set_popover(Some(&popover_menu));

        // wire everything up
        get_widget!(self_.window_builder, gtk::Box, explore_box);
        explore_box.add(&app_private.explore_page.widget);

        get_widget!(self_.window_builder, gtk::Box, installed_box);
        installed_box.add(&app_private.installed_page.widget);

        get_widget!(self_.window_builder, gtk::Box, search_box);
        search_box.add(&app_private.search_page.widget);

        get_widget!(self_.window_builder, gtk::Box, package_details_box);
        package_details_box.add(&app_private.package_details_page.widget);

        // Add headerbar/content to the window itself
        get_widget!(self_.window_builder, gtk::Box, window);
        self.add(&window);
    }

    fn setup_signals(&self, sender: Sender<Action>) {
        let self_ = GsApplicationWindowPrivate::from_instance(self);

        // main stack
        get_widget!(self_.window_builder, gtk::Stack, main_stack);
        main_stack.connect_property_visible_child_notify(
            clone!(@strong self as this => move |main_stack| {
                let view = match main_stack.get_visible_child_name().unwrap().as_str(){
                    "explore" => View::Explore,
                    "installed" => View::Installed,
                    "updates" => View::Updates,
                    "search" => View::Search,
                    _ => View::Explore,
                };
                this.set_view(view, false);
            }),
        );

        // back button (mouse)
        self.connect_button_press_event(clone!(@strong sender => move |_, event|{
            if event.get_button() == 8 {
                send!(sender, Action::ViewGoBack);
            }
            Inhibit(false)
        }));
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from GsApplicationWindow to libhandy::ApplicationWindow, because GsApplicationWindow
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

        // win.about
        action!(
            window,
            "about",
            clone!(@weak window => move |_, _| {
                about_dialog::show_about_dialog(window);
            })
        );

        // win.show-search
        action!(
            window,
            "show-search",
            clone!(@weak window, @strong sender => move |_, _| {
                send!(sender, Action::ViewSet(View::Search));
            })
        );
        app.set_accels_for_action("win.show-search", &["<primary>f"]);

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

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let self_ = GsApplicationWindowPrivate::from_instance(self);

        // Remove current page
        let _ = self_.pages_stack.borrow_mut().pop();

        // Get previous page and set it as current view
        let view = self_
            .pages_stack
            .borrow()
            .last()
            .unwrap_or(&View::Explore)
            .clone();
        self.set_view(view, true);
    }

    pub fn set_view(&self, view: View, go_back: bool) {
        debug!("Set view to {:?}", &view);

        let self_ = GsApplicationWindowPrivate::from_instance(self);
        let app: GsApplication = self
            .get_application()
            .unwrap()
            .downcast::<GsApplication>()
            .unwrap();
        let app_private = GsApplicationPrivate::from_instance(&app);

        get_widget!(self_.window_builder, libhandy::Deck, window_deck);
        get_widget!(self_.window_builder, gtk::Stack, main_stack);

        // Show requested view / page
        match view.clone() {
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
            View::Search => {
                main_stack.set_visible_child_name("search");
                window_deck.set_visible_child_name("main");
            }
            View::PackageDetails(package) => {
                window_deck.set_visible_child_name("package-details");
                app_private.package_details_page.reset();
                app_private.package_details_page.set_package(*package);
            }
        }

        if !go_back
            && view != View::Explore
            && view != View::Installed
            && view != View::Updates
            && view != View::Search
        {
            self_.pages_stack.borrow_mut().push(view);
        }
        dbg!(self_.pages_stack.borrow());
    }
}
