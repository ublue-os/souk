use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{SoukPackage, SoukPackageKind};
use crate::ui::package_widgets::PackageWidget;

pub struct PackageActionButton {
    pub widget: gtk::Box,
    package: Rc<RefCell<Option<SoukPackage>>>,

    builder: gtk::Builder,
}

impl PackageActionButton {
    fn setup_signals(&self) {
        // install
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            //this.flatpak_backend.clone().install_package(&this.package);
        }));

        // uninstall
        get_widget!(self.builder, gtk::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            debug!("Uninstall");
            //this.flatpak_backend.clone().uninstall_package(&this.package);
        }));

        // open
        get_widget!(self.builder, gtk::Button, open_button);
        open_button.connect_clicked(clone!(@weak self.package as package => move |_|{
            //this.flatpak_backend.clone().launch_package(&this.package);
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

    /*
    async fn receive_transaction_messages(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, button_stack);
        get_widget!(self.builder, gtk::ProgressBar, progressbar);
        get_widget!(self.builder, gtk::Label, status_label);

        let mut transaction_channel = self.transaction.borrow().as_ref().unwrap().get_channel();
        button_stack.set_visible_child_name("processing");

        // TODO: Don't show this message when installing packages.
        // It is currently being displayed for ~20ms.
        status_label.set_text("Working…");
        // Listen to transaction state changes
        while let Some(state) = transaction_channel.recv().await {
            progressbar.set_fraction(state.percentage.into());
            if &state.message != "" {
                status_label.set_text(&state.message);
            }

            match state.mode {
                TransactionMode::Finished | TransactionMode::Cancelled => {
                    status_label.set_text("");
                    self.clone().update_stack();
                    break;
                }
                TransactionMode::Error(err) => {
                    status_label.set_text("");
                    self.clone().update_stack();
                    utils::show_error_dialog(&err);
                    break;
                }
                _ => (),
            };
        }

        *self.transaction.borrow_mut() = None;
    }*/

    fn update_stack(&self) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        /*match self
            .flatpak_backend
            .clone()
            .is_package_installed(&self.package)
        {
            true => {
                button_stack.set_visible_child_name("installed");
            }
            false => {
                button_stack.set_visible_child_name("install");
            }
        };*/
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
            builder,
        };

        pab.update_stack();
        pab.setup_signals();
        pab
    }

    fn set_package(&self, package: &SoukPackage) {
        *self.package.borrow_mut() = Some(package.clone());

        // Hide open button for runtimes and extensions
        if package.get_kind() != SoukPackageKind::App {
            get_widget!(self.builder, gtk::Button, open_button);
            open_button.set_visible(false);
        }
    }

    fn reset(&self) {}
}
