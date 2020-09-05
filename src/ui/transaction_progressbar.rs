use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::{FlatpakBackend, BackendMessage, Package, PackageTransaction};
use crate::ui::utils;

pub struct TransactionProgressBar {
    pub widget: gtk::Box,
    package: Package,

    flatpak_backend: Rc<FlatpakBackend>,
    builder: gtk::Builder,
}

impl TransactionProgressBar {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>, package: Package) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/transaction_progressbar.ui");
        get_widget!(builder, gtk::Box, transaction_progressbar);

        let transaction_progressbar = Self {
            widget: transaction_progressbar,
            package,
            flatpak_backend,
            builder,
        };

        transaction_progressbar.setup_signals();
        transaction_progressbar
    }

    fn setup_signals(&self) {
        spawn!(Self::backend_message_receiver(self.builder.clone(), self.package.clone(), self.flatpak_backend.clone()));
    }

    async fn backend_message_receiver(builder: gtk::Builder, package: Package, flatpak_backend: Rc<FlatpakBackend>) {
        get_widget!(builder, gtk::ProgressBar, progressbar);
        get_widget!(builder, gtk::Revealer, progressbar_revealer);
        get_widget!(builder, gtk::Label, log_label);
        get_widget!(builder, gtk::ScrolledWindow, log_scrolled_window);

        let mut backend_channel = flatpak_backend.clone().get_channel();

        while let Some(backend_message) = backend_channel.recv().await {
            match backend_message{
                BackendMessage::NewPackageTransaction(transaction) => {
                    // We only care about this package
                    if transaction.package == package {
                        let mut transaction_channel = transaction.clone().get_channel();

                        // We have a transaction which affects this package, so display progressbar
                        progressbar_revealer.set_reveal_child(true);

                        // Listen to transaction state changes
                        while let Some(state) = transaction_channel.recv().await {
                            progressbar.set_fraction(state.percentage.into());
                            let text = format!("{}\n{}", log_label.get_text(), state.message);
                            log_label.set_text(&text);

                            // scroll down
                            match log_scrolled_window.get_vadjustment(){
                                Some(adj) => adj.set_value(adj.get_upper() - adj.get_page_size()),
                                None => (),
                            };

                            if state.is_finished {
                                break;
                            }
                        }
                    }
                },
                _ => (),
            }
        }
    }
}
