use gio::prelude::*;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk4::prelude::*;
use libhandy4::auto::traits::*;

use std::cell::RefCell;

use crate::app::{Action, GsApplication, GsApplicationPrivate};
use crate::backend::Package;
use crate::ui::about_dialog;

#[derive(Debug, Clone)]
pub enum View {
    Explore,
    Installed,
    Updates,
    Search,
    PackageDetails(Box<dyn Package>),
}

pub struct ApplicationWindow {
    pub widget: libhandy4::ApplicationWindow,

    window_builder: gtk4::Builder,
    menu_builder: gtk4::Builder,

    pages_stack: RefCell<Vec<View>>,
}

impl ApplicationWindow {
    pub fn new(sender: Sender<Action>, app: GsApplication) -> Self {
        let window_builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/window.ui");
        let menu_builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/menu.ui");

        get_widget!(window_builder, libhandy4::ApplicationWindow, window);
        let pages_stack = RefCell::new(Vec::new());

        app.add_window(&window);

        let application_window = Self {
            widget: window,
            window_builder,
            menu_builder,
            pages_stack,
        };

        application_window.setup_widgets();
        application_window.setup_signals(sender.clone());
        application_window.setup_gactions(sender);
        application_window
    }

    pub fn setup_widgets(&self) {
        let app: GsApplication = self
            .widget
            .get_application()
            .unwrap()
            .downcast::<GsApplication>()
            .unwrap();
        let app_private = GsApplicationPrivate::from_instance(&app);

        // set default size
        self.widget.set_default_size(900, 700);

        // Set hamburger menu
        get_widget!(self.menu_builder, gtk4::PopoverMenu, popover_menu);
        get_widget!(self.window_builder, gtk4::MenuButton, appmenu_button);
        appmenu_button.set_popover(Some(&popover_menu));

        // wire everything up
        get_widget!(self.window_builder, gtk4::Box, explore_box);
        explore_box.append(&app_private.explore_page.widget);

        get_widget!(self.window_builder, gtk4::Box, installed_box);
        installed_box.append(&app_private.installed_page.widget);

        get_widget!(self.window_builder, gtk4::Box, search_box);
        search_box.append(&app_private.search_page.widget);

        get_widget!(self.window_builder, gtk4::Box, package_details_box);
        package_details_box.append(&app_private.package_details_page.widget);
    }

    fn setup_signals(&self, sender: Sender<Action>) {
        // main stack
        // TODO: fixme...
        /*
        get_widget!(self.window_builder, gtk4::Stack, main_stack);
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
        );*/

        // back button (mouse)
        // TODO: Fixme!
        /*self.widget.connect_button_press_event(clone!(@strong sender => move |_, event|{
            if event.get_button() == 8 {
                send!(sender, Action::ViewGoBack);
            }
            glib::signal::Inhibit(false)
        }));*/
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from GsApplicationWindow to libhandy4::ApplicationWindow, because GsApplicationWindow
        // currently doesn't implement GLib.ActionMap, since it's not supported in gtk-rs for subclassing (13-01-2020)
        let window = self.widget.clone().upcast::<gtk4::ApplicationWindow>();
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

        // Remove current page
        let _ = self.pages_stack.borrow_mut().pop();

        // Get previous page and set it as current view
        let view = self
            .pages_stack
            .borrow()
            .last()
            .unwrap_or(&View::Explore)
            .clone();
        self.set_view(view, true);
    }

    pub fn set_view(&self, view: View, go_back: bool) {
        debug!("Set view to {:?}", &view);

        let app: GsApplication = self
            .widget
            .get_application()
            .unwrap()
            .downcast::<GsApplication>()
            .unwrap();
        let app_private = GsApplicationPrivate::from_instance(&app);

        get_widget!(self.window_builder, gtk4::Stack, window_stack);
        get_widget!(self.window_builder, gtk4::Stack, main_stack);

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
                app_private.package_details_page.reset();
                app_private.package_details_page.set_package(&*package);
            }
        }

        // Don't add page to pages stack, when we're going back
        if !go_back {
            self.pages_stack.borrow_mut().push(view.clone());
        }

        // It doesn't make sense to track changes between Explore / Installed / Updates,
        // since they're at main "root" view where it isn't possible to go back.
        match view {
            View::Explore | View::Installed | View::Updates => {
                self.pages_stack.borrow_mut().clear();
                self.pages_stack.borrow_mut().push(view);
            }
            _ => (),
        }
    }
}
