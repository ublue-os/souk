use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;
use std::sync::Arc;

use crate::app::Action;
use crate::backend::{SoukFlatpakBackend, SoukPackage, SoukPackageKind, TransactionMode};
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
        get_widget!(self.builder, gtk::ListBox, listbox_runtimes);

        let model: gio::ListStore = self
            .flatpak_backend
            .get_property("installed_packages")
            .unwrap()
            .get()
            .unwrap()
            .unwrap();

        // Apps section
        let apps_filter = gtk::CustomFilter::new(Some(Box::new(|object| {
            let package = object.clone().downcast::<SoukPackage>().unwrap();
            let kind: SoukPackageKind = package
                .get_property("kind")
                .unwrap()
                .get()
                .unwrap()
                .unwrap();

            kind == SoukPackageKind::App
        })));
        let apps_model = gtk::FilterListModel::new(Some(&model), Some(&apps_filter));

        listbox_apps.bind_model(
            Some(&apps_model),
            Some(Box::new(|package| {
                let row = SoukPackageRow::new();
                row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
                row.upcast::<gtk::Widget>()
            })),
        );

        // Runtimes section
        let runtimes_filter = gtk::CustomFilter::new(Some(Box::new(|object| {
            let package = object.clone().downcast::<SoukPackage>().unwrap();
            let kind: SoukPackageKind = package
                .get_property("kind")
                .unwrap()
                .get()
                .unwrap()
                .unwrap();

            kind == SoukPackageKind::Runtime
        })));
        let runtimes_model = gtk::FilterListModel::new(Some(&model), Some(&runtimes_filter));

        listbox_runtimes.bind_model(
            Some(&runtimes_model),
            Some(Box::new(|package| {
                let row = SoukPackageRow::new();
                row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
                row.upcast::<gtk::Widget>()
            })),
        );
    }
}
