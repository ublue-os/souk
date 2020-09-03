use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::{BackendMessage, FlatpakBackend, PackageTransaction, TransactionState};
use crate::ui::AppTile;

pub struct InstalledPage {
    pub widget: gtk::Box,
    flatpak_backend: Rc<FlatpakBackend>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl InstalledPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: Rc<FlatpakBackend>) -> Rc<Self> {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/FlatpakFrontend/gtk/installed_page.ui");
        get_widget!(builder, gtk::Box, installed_page);

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
        get_widget!(self.builder, gtk::FlowBox, installed_flowbox);

        let packages = self.flatpak_backend.clone().get_installed_packages();
        for package in packages {
            debug!("Installed package: {:?}", &package);
            let tile = AppTile::new(self.sender.clone(), package);
            installed_flowbox.add(&tile.widget);
        }

        installed_flowbox.show_all();
    }

    fn setup_signals(self: Rc<Self>) {
        spawn!(self.message_loop());
    }

    async fn message_loop(self: Rc<Self>) {
        let mut channel = self.flatpak_backend.clone().get_channel();
        get_widget!(self.builder, gtk::Label, transaction_label);

        while let Some(message) = channel.recv().await {
            debug!("message: {:?}", &message);
            transaction_label.set_text(&format!("{:#?}", &message));
        }
    }
}
