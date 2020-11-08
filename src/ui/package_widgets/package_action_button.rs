use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{SoukPackage, SoukPackageKind, SoukTransactionMode};
use crate::ui::package_widgets::PackageWidget;

pub struct PackageActionButton {
    pub widget: gtk::Box,

    package: Rc<RefCell<Option<SoukPackage>>>,
    state_signal_id: RefCell<Option<glib::SignalHandlerId>>,
    installed_signal_id: RefCell<Option<glib::SignalHandlerId>>,

    builder: gtk::Builder,
}

impl PackageActionButton {
    fn setup_signals(&self) {
        // install
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            package.borrow().as_ref().unwrap().install();
        }));

        // uninstall
        get_widget!(self.builder, gtk::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            package.borrow().as_ref().unwrap().uninstall();
        }));

        // open
        get_widget!(self.builder, gtk::Button, open_button);
        open_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            package.borrow().as_ref().unwrap().launch();
        }));

        // cancel
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            //match this.transaction.borrow().clone(){
            //    Some(t) => (), //this.flatpak_backend.clone().cancel_package_transaction(t),
            //    None => warn!("No transaction available to cancel"),
            //};
        }));
    }

    fn update_stack(builder: gtk::Builder, package: SoukPackage) {
        get_widget!(builder, gtk::Stack, button_stack);
        get_widget!(builder, gtk::Label, status_label);
        get_widget!(builder, gtk::ProgressBar, progressbar);

        match package.get_transaction_state() {
            Some(state) => {
                button_stack.set_visible_child_name("processing");
                progressbar.set_fraction(state.get_percentage().into());
                if &state.get_message() != "" {
                    status_label.set_text(&state.get_message());
                }

                match state.get_mode() {
                    SoukTransactionMode::Finished | SoukTransactionMode::Cancelled => {
                        status_label.set_text("");
                    }
                    SoukTransactionMode::Error => {
                        status_label.set_text("");
                        //utils::show_error_dialog(&err);
                    }
                    _ => (),
                };
            }
            None => {
                status_label.set_text("");
                if package.get_installed_info().is_some() {
                    button_stack.set_visible_child_name("installed");
                } else {
                    button_stack.set_visible_child_name("install");
                }
            }
        }
    }
}

impl PackageWidget for PackageActionButton {
    fn new() -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_action_button.ui");
        get_widget!(builder, gtk::Box, package_action_button);

        let pab = Self {
            widget: package_action_button,
            package: Rc::default(),
            state_signal_id: RefCell::default(),
            installed_signal_id: RefCell::default(),
            builder,
        };

        pab.setup_signals();
        pab
    }

    fn set_package(&self, package: &SoukPackage) {
        // Disconnect from previous package signals
        if let Some(id) = self.state_signal_id.borrow_mut().take() {
            self.package.borrow().as_ref().unwrap().disconnect(id);
        }
        if let Some(id) = self.installed_signal_id.borrow_mut().take() {
            self.package.borrow().as_ref().unwrap().disconnect(id);
        }

        Self::update_stack(self.builder.clone(), package.clone());

        let closure = clone!(@weak self.builder as builder, @weak package => @default-return None::<glib::Value>, move |_:&[glib::Value]|{
            Self::update_stack(builder.clone(), package.clone());
            None
        });

        // Listen to transaction state changes...
        let state_signal_id = package
            .connect_local("notify::transaction-state", false, closure.clone())
            .unwrap();
        *self.state_signal_id.borrow_mut() = Some(state_signal_id);

        // Listen to installed changes...
        let installed_signal_id = package
            .connect_local("notify::installed-info", false, closure.clone())
            .unwrap();
        *self.installed_signal_id.borrow_mut() = Some(installed_signal_id);

        *self.package.borrow_mut() = Some(package.clone());

        // Hide open button for runtimes and extensions
        if package.get_kind() != SoukPackageKind::App {
            get_widget!(self.builder, gtk::Button, open_button);
            open_button.set_visible(false);
        }
    }

    fn reset(&self) {}
}
