use gtk4::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::backend::{
    BackendMessage, BasePackage, FlatpakBackend, Package, PackageKind, PackageTransaction,
    TransactionMode,
};
use crate::ui::utils;

pub struct PackageActionButton {
    pub widget: gtk4::Box,
    package: BasePackage,
    transaction: RefCell<Option<Arc<PackageTransaction>>>,

    flatpak_backend: Rc<FlatpakBackend>,
    builder: gtk4::Builder,
}

impl PackageActionButton {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>, package: &dyn Package) -> Rc<Self> {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/package_action_button.ui");
        get_widget!(builder, gtk4::Box, package_action_button);
        let transaction = RefCell::new(None);

        let pab = Rc::new(Self {
            widget: package_action_button,
            package: package.base_package().clone(),
            transaction,
            flatpak_backend,
            builder,
        });

        // Check if a transcation is already running
        match pab
            .flatpak_backend
            .clone()
            .get_active_transaction(&pab.package)
        {
            Some(t) => {
                debug!("Found already running transaction - display state for it.");
                *pab.transaction.borrow_mut() = Some(t);
                spawn!(pab.clone().receive_transaction_messages());
            }
            None => (),
        }

        // Hide open button for runtimes and extensions
        if package.kind() != PackageKind::App {
            get_widget!(pab.builder, gtk4::Button, open_button);
            open_button.set_visible(false);
        }

        pab.clone().update_stack();
        pab.clone().setup_signals();
        pab
    }

    fn setup_signals(self: Rc<Self>) {
        // install
        get_widget!(self.builder, gtk4::Button, install_button);
        install_button.connect_clicked(clone!(@weak self as this => move |_|{
            this.flatpak_backend.clone().install_package(&this.package);
        }));

        // uninstall
        get_widget!(self.builder, gtk4::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@weak self as this => move |_|{
            debug!("Uninstall");
            this.flatpak_backend.clone().uninstall_package(&this.package);
        }));

        // open
        get_widget!(self.builder, gtk4::Button, open_button);
        open_button.connect_clicked(clone!(@weak self as this => move |_|{
            this.flatpak_backend.clone().launch_package(&this.package);
        }));

        // cancel
        get_widget!(self.builder, gtk4::Button, cancel_button);
        cancel_button.connect_clicked(clone!(@weak self as this => move |_|{
            match this.transaction.borrow().clone(){
                Some(t) => this.flatpak_backend.clone().cancel_package_transaction(t),
                None => warn!("No transaction available to cancel"),
            };
        }));

        spawn!(self.receive_backend_messages());
    }

    async fn receive_backend_messages(self: Rc<Self>) {
        let mut backend_channel = self.flatpak_backend.clone().get_channel();

        while let Some(backend_message) = backend_channel.recv().await {
            match backend_message {
                BackendMessage::PackageTransaction(transaction) => {
                    if transaction.package == self.package {
                        *self.transaction.borrow_mut() = Some(transaction.clone());
                        self.clone().receive_transaction_messages().await;
                    }
                }
            }
        }
    }

    async fn receive_transaction_messages(self: Rc<Self>) {
        get_widget!(self.builder, gtk4::Stack, button_stack);
        get_widget!(self.builder, gtk4::ProgressBar, progressbar);
        get_widget!(self.builder, gtk4::Label, status_label);

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
                    utils::show_error_dialog(self.builder.clone(), &err);
                    break;
                }
                _ => (),
            };
        }

        *self.transaction.borrow_mut() = None;
    }

    fn update_stack(self: Rc<Self>) {
        get_widget!(self.builder, gtk4::Stack, button_stack);

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
