use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};
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

#[derive(Debug, CompositeTemplate)]
pub struct SoukApplicationWindowPrivate {
    #[template_child(id = "view_switcher_title")]
    pub view_switcher_title: TemplateChild<libhandy::ViewSwitcherTitle>,
    #[template_child(id = "explore_box")]
    pub explore_box: TemplateChild<gtk::Box>,
    #[template_child(id = "installed_box")]
    pub installed_box: TemplateChild<gtk::Box>,
    #[template_child(id = "search_box")]
    pub search_box: TemplateChild<gtk::Box>,
    #[template_child(id = "package_details_box")]
    pub package_details_box: TemplateChild<gtk::Box>,
    #[template_child(id = "main_stack")]
    pub main_stack: TemplateChild<gtk::Stack>,
    #[template_child(id = "window_stack")]
    pub window_stack: TemplateChild<gtk::Stack>,

    pages_stack: RefCell<Vec<View>>,
}

impl ObjectSubclass for SoukApplicationWindowPrivate {
    const NAME: &'static str = "SoukApplicationWindow";
    type Type = SoukApplicationWindow;
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/window.ui");
        Self::bind_template_children(klass);
    }

    glib_object_subclass!();

    fn new() -> Self {
        let pages_stack = RefCell::new(Vec::new());

        Self {
            view_switcher_title: TemplateChild::default(),
            explore_box: TemplateChild::default(),
            installed_box: TemplateChild::default(),
            search_box: TemplateChild::default(),
            package_details_box: TemplateChild::default(),
            main_stack: TemplateChild::default(),
            window_stack: TemplateChild::default(),
            pages_stack,
        }
    }
}

// Implement GLib.OBject for SoukApplicationWindow
impl ObjectImpl for SoukApplicationWindowPrivate {
    fn constructed(&self, obj: &Self::Type) {
        obj.init_template();
        self.parent_constructed(obj);
    }
}

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

        // set title
        self_
            .view_switcher_title
            .get()
            .set_title(Some(config::NAME));
        self.set_title(config::NAME);

        // wire everything up
        self_
            .explore_box
            .get()
            .append(&app_private.explore_page.get().unwrap().widget);
        self_
            .installed_box
            .get()
            .append(&app_private.installed_page.get().unwrap().widget);
        self_
            .search_box
            .get()
            .append(&app_private.search_page.get().unwrap().widget);
        self_
            .package_details_box
            .get()
            .append(&app_private.package_details_page.get().unwrap().widget);
    }

    fn setup_signals(&self) {
        let self_ = SoukApplicationWindowPrivate::from_instance(self);

        // main stack
        self_
            .main_stack
            .get()
            .connect_property_visible_child_notify(
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

        let main_stack = self_.main_stack.get();
        let window_stack = self_.window_stack.get();

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
