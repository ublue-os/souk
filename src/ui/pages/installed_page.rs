use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::{SoukFlatpakBackend, SoukPackage, SoukPackageKind};
use crate::ui::SoukPackageRow;
use crate::ui::View;

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
        installed_page.clone().setup_signals();
        installed_page
    }

    fn setup_widgets(self: Rc<Self>) {
        get_widget!(self.builder, gtk::ListBox, listbox_apps);
        get_widget!(self.builder, gtk::ListBox, listbox_runtimes);

        let model: gio::ListStore = self.flatpak_backend.get_installed_packages();

        // Apps section
        let apps_filter = gtk::CustomFilter::new(|object| {
            let package = object.clone().downcast::<SoukPackage>().unwrap();
            package.get_kind() == SoukPackageKind::App
        });
        let apps_model = gtk::FilterListModel::new(Some(&model), Some(&apps_filter));

        listbox_apps.bind_model(Some(&apps_model), |package| {
            let row = SoukPackageRow::new(true);
            row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
            row.upcast::<gtk::Widget>()
        });

        // Runtimes section
        let runtimes_filter = gtk::CustomFilter::new(|object| {
            let package = object.clone().downcast::<SoukPackage>().unwrap();
            package.get_kind() == SoukPackageKind::Runtime
        });
        let runtimes_model = gtk::FilterListModel::new(Some(&model), Some(&runtimes_filter));

        listbox_runtimes.bind_model(Some(&runtimes_model), |package| {
            let row = SoukPackageRow::new(true);
            row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
            row.upcast::<gtk::Widget>()
        });
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::ListBox, listbox_apps);
        get_widget!(self.builder, gtk::ListBox, listbox_runtimes);

        let closure = clone!(@weak self as this => move|_: &gtk::ListBox, listbox_row: &gtk::ListBoxRow|{
            let row = listbox_row.clone().downcast::<SoukPackageRow>().unwrap();
            let package: SoukPackage = row.get_package().unwrap();
            send!(this.sender, Action::ViewSet(View::PackageDetails(package)));
        });

        listbox_apps.connect_row_activated(closure.clone());
        listbox_runtimes.connect_row_activated(closure.clone());
    }
}
