use glib::subclass;
use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};

use std::cell::RefCell;

use crate::backend::{SoukPackage, SoukPackageKind, SoukTransactionMode};
use crate::ui::package_widgets::PackageWidget;
use crate::ui::utils;

#[derive(Debug, CompositeTemplate)]
pub struct SoukActionButtonPrivate {
    #[template_child(id = "button_stack")]
    pub button_stack: TemplateChild<gtk::Stack>,
    #[template_child(id = "status_label")]
    pub status_label: TemplateChild<gtk::Label>,
    #[template_child(id = "progressbar")]
    pub progressbar: TemplateChild<gtk::ProgressBar>,
    #[template_child(id = "install_button")]
    pub install_button: TemplateChild<gtk::Button>,
    #[template_child(id = "uninstall_button")]
    pub uninstall_button: TemplateChild<gtk::Button>,
    #[template_child(id = "open_button")]
    open_button: TemplateChild<gtk::Button>,
    #[template_child(id = "cancel_button")]
    cancel_button: TemplateChild<gtk::Button>,

    package: RefCell<Option<SoukPackage>>,
    state_signal_id: RefCell<Option<glib::SignalHandlerId>>,
    installed_signal_id: RefCell<Option<glib::SignalHandlerId>>,
}

impl ObjectSubclass for SoukActionButtonPrivate {
    const NAME: &'static str = "SoukActionButton";
    type Type = SoukActionButton;
    type ParentType = gtk::Box;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/action_button.ui");
        Self::bind_template_children(klass);
    }

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            button_stack: TemplateChild::default(),
            status_label: TemplateChild::default(),
            progressbar: TemplateChild::default(),
            install_button: TemplateChild::default(),
            uninstall_button: TemplateChild::default(),
            open_button: TemplateChild::default(),
            cancel_button: TemplateChild::default(),
            package: RefCell::default(),
            state_signal_id: RefCell::default(),
            installed_signal_id: RefCell::default(),
        }
    }
}

impl ObjectImpl for SoukActionButtonPrivate {
    fn constructed(&self, obj: &Self::Type) {
        obj.init_template();
        self.parent_constructed(obj);
    }
}

impl WidgetImpl for SoukActionButtonPrivate {}

impl BoxImpl for SoukActionButtonPrivate {}

glib_wrapper! {
    pub struct SoukActionButton(ObjectSubclass<SoukActionButtonPrivate>)
    @extends gtk::Widget, gtk::Box;
}

impl SoukActionButton {
    fn setup_signals(&self) {
        let self_ = SoukActionButtonPrivate::from_instance(self);

        // install
        self_
            .install_button
            .get()
            .connect_clicked(clone!(@strong self as this => move |_|{
                let self_ = SoukActionButtonPrivate::from_instance(&this);
                self_.package.borrow().as_ref().unwrap().install();
            }));

        // uninstall
        self_
            .uninstall_button
            .get()
            .connect_clicked(clone!(@strong self as this => move |_|{
                let self_ = SoukActionButtonPrivate::from_instance(&this);
                self_.package.borrow().as_ref().unwrap().uninstall();
            }));

        // open
        self_
            .open_button
            .get()
            .connect_clicked(clone!(@strong self as this => move |_|{
                let self_ = SoukActionButtonPrivate::from_instance(&this);
                self_.package.borrow().as_ref().unwrap().launch();
            }));

        // cancel
        self_
            .cancel_button
            .get()
            .connect_clicked(clone!(@strong self as this => move |_|{
                let self_ = SoukActionButtonPrivate::from_instance(&this);
                self_.package.borrow().as_ref().unwrap().cancel_transaction();
            }));
    }

    fn update_stack(&self) {
        let self_ = SoukActionButtonPrivate::from_instance(self);
        let state = self_
            .package
            .borrow()
            .as_ref()
            .unwrap()
            .get_transaction_state();

        // Check if transaction is active / running
        if state.get_mode() == SoukTransactionMode::Running {
            // Show correct button
            self_
                .button_stack
                .get()
                .set_visible_child_name("processing");

            // Set progressbar fraction
            self_
                .progressbar
                .get()
                .set_fraction(state.get_percentage().into());

            // Set transaction message
            if &state.get_message() != "" {
                self_.status_label.get().set_text(&state.get_message());
            }
        } else {
            // Transaction isn't running anymore -> check result
            if state.get_mode() == SoukTransactionMode::Error {
                utils::show_error_dialog("Someting went wrong");
                // TODO: Show proper error message here
            }
            self_.status_label.get().set_text("");

            // ... and show (un)install button again!
            if self_.package.borrow().as_ref().unwrap().get_is_installed() {
                self_.button_stack.get().set_visible_child_name("installed");
            } else {
                self_.button_stack.get().set_visible_child_name("install");
            }
        }
    }
}

impl PackageWidget for SoukActionButton {
    fn new() -> Self {
        let button = glib::Object::new(SoukActionButton::static_type(), &[])
            .unwrap()
            .downcast::<SoukActionButton>()
            .unwrap();

        button.setup_signals();
        button
    }

    fn set_package(&self, package: &SoukPackage) {
        let self_ = SoukActionButtonPrivate::from_instance(self);

        *self_.package.borrow_mut() = Some(package.clone());
        self.update_stack();

        let closure = clone!(@weak self as this => @default-panic, move |_:&[glib::Value]|{
            this.update_stack();
            None
        });

        // Listen to transaction state changes...
        let state_signal_id = package
            .connect_local("notify::transaction-state", false, closure.clone())
            .unwrap();
        *self_.state_signal_id.borrow_mut() = Some(state_signal_id);

        // Listen to installed changes...
        let installed_signal_id = package
            .connect_local("notify::installed-info", false, closure.clone())
            .unwrap();
        *self_.installed_signal_id.borrow_mut() = Some(installed_signal_id);

        // Hide open button for runtimes and extensions
        if package.get_kind() != SoukPackageKind::App {
            self_.open_button.get().set_visible(false);
        }
    }

    fn reset(&self) {
        let self_ = SoukActionButtonPrivate::from_instance(self);

        if let Some(id) = self_.state_signal_id.borrow_mut().take() {
            self_.package.borrow().as_ref().unwrap().disconnect(id);
        }

        if let Some(id) = self_.installed_signal_id.borrow_mut().take() {
            self_.package.borrow().as_ref().unwrap().disconnect(id);
        }
    }
}
