use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;
use std::sync::Arc;

use crate::app::Action;
use crate::backend::{PackageTransaction, SoukFlatpakBackend, SoukPackage, TransactionMode};
use crate::database::DisplayLevel;
use crate::ui::SoukPackageRow;

pub struct InstalledPage {
    pub widget: gtk::Box,
    flatpak_backend: SoukFlatpakBackend,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl InstalledPage {
    pub fn new(sender: Sender<Action>, flatpak_backend: SoukFlatpakBackend) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/installed_page.ui");
        get_widget!(builder, gtk::Box, installed_page);

        let installed_page = Rc::new(Self {
            widget: installed_page,
            flatpak_backend,
            builder,
            sender,
        });

        installed_page.clone().setup_widgets();
        installed_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::ListBox, listbox_apps);
        let model: gio::ListStore = self
            .flatpak_backend
            .get_property("installed_packages")
            .unwrap()
            .get()
            .unwrap()
            .unwrap();

        listbox_apps.bind_model(
            Some(&model),
            Some(Box::new(|package| {
                let row = SoukPackageRow::new();
                row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
                row.upcast::<gtk::Widget>()
            })),
        );
    }
}
