use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::backend::{
    BackendMessage, FlatpakBackend, Package, PackageTransaction, TransactionMode,
};
use crate::ui::utils;

pub struct PackageActionButton {
    pub widget: gtk::Box,
    package: Package,
    transaction: RefCell<Option<Arc<PackageTransaction>>>,

    flatpak_backend: Rc<FlatpakBackend>,
    builder: gtk::Builder,
}

impl PackageActionButton {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>, package: Package) -> Rc<Self> {
        let builder = gtk::Builder::from_resource(
            "/de/haeckerfelix/FlatpakFrontend/gtk/package_action_button.ui",
        );
        get_widget!(builder, gtk::Box, package_action_button);
        let transaction = RefCell::new(None);

        let package_action_button = Rc::new(Self {
            widget: package_action_button,
            package,
            transaction,
            flatpak_backend,
            builder,
        });

        package_action_button.clone().update_stack();
        package_action_button.clone().setup_signals();
        package_action_button
    }

    fn setup_signals(self: Rc<Self>) {
        // install
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@weak self as this => move |_|{
            this.flatpak_backend.clone().install_package(this.package.clone());
        }));

        // uninstall
        get_widget!(self.builder, gtk::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@weak self as this => move |_|{
            debug!("Uninstall");
            this.flatpak_backend.clone().uninstall_package(this.package.clone());
        }));

        // open
        get_widget!(self.builder, gtk::Button, open_button);
        open_button.connect_clicked(clone!(@weak self as this => move |_|{
            this.flatpak_backend.clone().launch_package(this.package.clone());
        }));

        // cancel
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(clone!(@weak self as this => move |_|{
            match this.transaction.borrow().clone(){
                Some(t) => this.flatpak_backend.clone().cancel_package_transaction(t.clone()),
                None => warn!("No transaction available to cancel"),
            };
        }));

        spawn!(self.receive_backend_messages());
    }

    async fn receive_backend_messages(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, button_stack);
        get_widget!(self.builder, gtk::ProgressBar, progressbar);
        get_widget!(self.builder, gtk::Label, status_label);

        let mut backend_channel = self.flatpak_backend.clone().get_channel();

        while let Some(backend_message) = backend_channel.recv().await {
            match backend_message {
                BackendMessage::PackageTransaction(transaction) => {
                    if transaction.package == self.package {
                        *self.transaction.borrow_mut() = Some(transaction.clone());

                        let mut transaction_channel = transaction.clone().get_channel();
                        button_stack.set_visible_child_name("processing");

                        // Listen to transaction state changes
                        while let Some(state) = transaction_channel.recv().await {
                            progressbar.set_fraction(state.percentage.into());
                            status_label.set_text(&state.message);

                            match state.mode {
                                TransactionMode::Finished | TransactionMode::Cancelled => {
                                    status_label.set_text("");
                                    self.clone().update_stack();
                                    break;
                                }
                                TransactionMode::Error(err) => {
                                    status_label.set_text("");
                                    self.clone().update_stack();
                                    utils::show_error_dialog(self.builder.clone(), &err);
                                    break;
                                }
                                _ => (),
                            };
                        }

                        *self.transaction.borrow_mut() = None;
                    }
                }
            }
        }
    }

    fn update_stack(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        match self
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
        };
    }
}
