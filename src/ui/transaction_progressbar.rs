use gtk::prelude::*;

use std::rc::Rc;

use crate::backend::{BackendMessage, FlatpakBackend, Package};

pub struct TransactionProgressBar {
    pub widget: gtk::Box,
    package: Package,

    flatpak_backend: Rc<FlatpakBackend>,
    builder: gtk::Builder,
}

impl TransactionProgressBar {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>, package: Package) -> Rc<Self> {
        let builder = gtk::Builder::from_resource(
            "/de/haeckerfelix/FlatpakFrontend/gtk/transaction_progressbar.ui",
        );
        get_widget!(builder, gtk::Box, transaction_progressbar);

        let transaction_progressbar = Rc::new(Self {
            widget: transaction_progressbar,
            package,
            flatpak_backend,
            builder,
        });

        transaction_progressbar.clone().setup_signals();
        transaction_progressbar
    }

    fn setup_signals(self: Rc<Self>) {
        spawn!(self.clone().receive_backend_messages());
    }

    async fn receive_backend_messages(self: Rc<Self>) {
        get_widget!(self.builder, gtk::ProgressBar, progressbar);
        get_widget!(self.builder, gtk::Revealer, progressbar_revealer);
        get_widget!(self.builder, gtk::Label, log_label);
        get_widget!(self.builder, gtk::ScrolledWindow, log_scrolled_window);

        let mut backend_channel = self.flatpak_backend.clone().get_channel();

        while let Some(backend_message) = backend_channel.recv().await {
            match backend_message {
                BackendMessage::PackageTransaction(transaction) => {
                    if transaction.package == self.package {
                        let mut transaction_channel = transaction.clone().get_channel();

                        // We have a transaction which affects this package, so display progressbar
                        progressbar_revealer.set_reveal_child(true);

                        // Listen to transaction state changes
                        while let Some(state) = transaction_channel.recv().await {
                            progressbar.set_fraction(state.percentage.into());
                            let text = format!("{}\n{}", log_label.get_text(), state.message);
                            log_label.set_text(&text);

                            // scroll down
                            match log_scrolled_window.get_vadjustment() {
                                Some(adj) => adj.set_value(adj.get_upper() - adj.get_page_size()),
                                None => (),
                            };

                            if state.is_finished {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
