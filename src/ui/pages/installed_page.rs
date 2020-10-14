use glib::Sender;
use gtk4::prelude::*;

use std::rc::Rc;
use std::sync::Arc;

use crate::app::Action;
use crate::backend::{BackendMessage, FlatpakBackend, PackageTransaction, TransactionMode};
use crate::database::DisplayLevel;
use crate::ui::PackageTile;

pub struct InstalledPage {
    pub widget: gtk4::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    builder: gtk4::Builder,
    sender: Sender<Action>,
}

impl InstalledPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder = gtk4::Builder::from_resource("/org/gnome/Store/gtk/installed_page.ui");
        get_widget!(builder, gtk4::Box, installed_page);

        let installed_page = Rc::new(Self {
            widget: installed_page,
            flatpak_backend,
            builder,
            sender,
        });

        installed_page.clone().setup_widgets();
        installed_page.clone().setup_signals();
        installed_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk4::FlowBox, installed_flowbox);

        let packages = self
            .flatpak_backend
            .clone()
            .get_installed_packages(DisplayLevel::Runtimes);
        for package in packages {
            let tile = PackageTile::new(self.sender.clone(), &package);
            installed_flowbox.insert(&tile.widget, 0);
        }
    }

    fn setup_signals(self: Rc<Self>) {
        spawn!(self.backend_message_receiver());
    }

    async fn backend_message_receiver(self: Rc<Self>) {
        let mut channel = self.flatpak_backend.clone().get_channel();

        while let Some(message) = channel.recv().await {
            match message {
                BackendMessage::PackageTransaction(transaction) => {
                    spawn!(self.clone().package_transaction_receiver(transaction));
                }
            }
        }
    }

    async fn package_transaction_receiver(self: Rc<Self>, transaction: Arc<PackageTransaction>) {
        let mut channel = transaction.clone().get_channel();

        while let Some(state) = channel.recv().await {
            // TODO: implement UI
            if state.mode == TransactionMode::Finished || state.mode == TransactionMode::Cancelled {
                break;
            }
        }
    }
}
