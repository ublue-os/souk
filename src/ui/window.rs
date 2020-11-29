use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::{WidgetImpl, WindowImpl};
use libhandy::prelude::*;

use std::cell::RefCell;

use crate::app::{SoukApplication, SoukApplicationPrivate};
use crate::backend::SoukPackage;
use crate::config;

#[derive(Debug, Clone)]
pub enum View {
    Explore,
    Installed,
    Updates,
    Search,
    PackageDetails(SoukPackage),
}

pub struct SoukApplicationWindowPrivate {
    window_builder: gtk::Builder,
    menu_builder: gtk::Builder,

    pages_stack: RefCell<Vec<View>>,
}

impl ObjectSubclass for SoukApplicationWindowPrivate {
    const NAME: &'static str = "SoukApplicationWindow";
    type Type = SoukApplicationWindow;
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let window_builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/window.ui");
        let menu_builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/menu.ui");

        let pages_stack = RefCell::new(Vec::new());

        Self {
            window_builder,
            menu_builder,
            pages_stack,
        }
    }
}

// Implement GLib.OBject for SoukApplicationWindow
impl ObjectImpl for SoukApplicationWindowPrivate {}

// Implement Gtk.Widget for SoukApplicationWindow
impl WidgetImpl for SoukApplicationWindowPrivate {}

// Implement Gtk.Window for SoukApplicationWindow
impl WindowImpl for SoukApplicationWindowPrivate {}

// Implement Gtk.ApplicationWindow for SoukApplicationWindow
impl gtk::subclass::prelude::ApplicationWindowImpl for SoukApplicationWindowPrivate {}

// Implement Hdy.ApplicationWindow for SoukApplicationWindow
impl libhandy::subclass::prelude::ApplicationWindowImpl for SoukApplicationWindowPrivate {}

// Wrap SoukApplicationWindowPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct SoukApplicationWindow(ObjectSubclass<SoukApplicationWindowPrivate>)
    @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow;
}

// SoukApplicationWindow implementation itself
impl SoukApplicationWindow {
    pub fn new(app: SoukApplication) -> Self {
        // Create new GObject and downcast it into SoukApplicationWindow
        let window = glib::Object::new(
            SoukApplicationWindow::static_type(),
            &[("application", &app)],
        )
        .unwrap()
        .downcast::<SoukApplicationWindow>()
        .unwrap();

        app.add_window(&window);
        window.setup_widgets();
        window.setup_signals();
        window
    }

    pub fn setup_widgets(&self) {
        let self_ = SoukApplicationWindowPrivate::from_instance(self);
        let app: SoukApplication = self
            .get_application()
            .unwrap()
            .downcast::<SoukApplication>()
            .unwrap();
        let app_private = SoukApplicationPrivate::from_instance(&app);

        // set default size
        self.set_default_size(900, 700);

        // set title
        get_widget!(
            self_.window_builder,
            libhandy::ViewSwitcherTitle,
            view_switcher_title
        );
        view_switcher_title.set_title(Some(config::NAME));
        self.set_title(config::NAME);

        // Set hamburger menu
        get_widget!(self_.window_builder, gtk::MenuButton, appmenu_button);
        get_widget!(self_.menu_builder, gio::MenuModel, primary_menu);
        appmenu_button.set_menu_model(Some(&primary_menu));

        // wire everything up
        get_widget!(self_.window_builder, gtk::Box, explore_box);
        explore_box.append(&app_private.explore_page.get().unwrap().widget);

        get_widget!(self_.window_builder, gtk::Box, installed_box);
        installed_box.append(&app_private.installed_page.get().unwrap().widget);

        get_widget!(self_.window_builder, gtk::Box, search_box);
        search_box.append(&app_private.search_page.get().unwrap().widget);

        get_widget!(self_.window_builder, gtk::Box, package_details_box);
        package_details_box.append(&app_private.package_details_page.get().unwrap().widget);

        // Add headerbar/content to the window itself
        get_widget!(self_.window_builder, gtk::Box, window);
        libhandy::ApplicationWindowExt::set_child(self, Some(&window));
    }

    fn setup_signals(&self) {
        let self_ = SoukApplicationWindowPrivate::from_instance(self);

        // main stack
        get_widget!(self_.window_builder, gtk::Stack, main_stack);
        main_stack.connect_property_visible_child_notify(
            clone!(@weak self as this => move |main_stack| {
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

        // TODO: back button (mouse)
        /* self.connect_button_press_event(clone!(@strong sender => move |_, event|{
            if event.get_button() == 8 {
                send!(sender, Action::ViewGoBack);
            }
            glib::signal::Inhibit(false)
        }));*/
    }

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let self_ = SoukApplicationWindowPrivate::from_instance(self);

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

        let self_ = SoukApplicationWindowPrivate::from_instance(self);
        let app: SoukApplication = self
            .get_application()
            .unwrap()
            .downcast::<SoukApplication>()
            .unwrap();
        let app_private = SoukApplicationPrivate::from_instance(&app);

        get_widget!(self_.window_builder, gtk::Stack, window_stack);
        get_widget!(self_.window_builder, gtk::Stack, main_stack);

        // Show requested view / page
        match view.clone() {
            View::Explore => {
                main_stack.set_visible_child_name("explore");
                window_stack.set_visible_child_name("main");
            }
            View::Installed => {
                main_stack.set_visible_child_name("installed");
                window_stack.set_visible_child_name("main");
            }
            View::Updates => {
                main_stack.set_visible_child_name("updates");
                window_stack.set_visible_child_name("main");
            }
            View::Search => {
                window_stack.set_visible_child_name("search");
            }
            View::PackageDetails(package) => {
                window_stack.set_visible_child_name("package-details");
                app_private.package_details_page.get().unwrap().reset();
                app_private
                    .package_details_page
                    .get()
                    .unwrap()
                    .set_package(package);
            }
        }

        // Don't add page to pages stack, when we're going back
        if !go_back {
            self_.pages_stack.borrow_mut().push(view.clone());
        }

        // It doesn't make sense to track changes between Explore / Installed / Updates,
        // since they're at main "root" view where it isn't possible to go back.
        match view {
            View::Explore | View::Installed | View::Updates => {
                self_.pages_stack.borrow_mut().clear();
                self_.pages_stack.borrow_mut().push(view);
            }
            _ => (),
        }
    }
}
