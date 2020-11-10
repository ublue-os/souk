use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::database::{queries, DisplayLevel};
use crate::ui::SoukPackageRow;
use crate::ui::View;

pub struct SearchPage {
    pub widget: gtk::Box,
    listview: gtk::ListView,

    model: gio::ListStore,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl SearchPage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/search_page.ui");
        get_widget!(builder, gtk::Box, search_page);
        get_widget!(builder, gtk::ListView, listview);

        let model = gio::ListStore::new(SoukPackage::static_type());
        let selection_model = gtk::NoSelection::new(Some(&model));
        listview.set_model(Some(&selection_model));

        let search_page = Rc::new(Self {
            widget: search_page,
            listview,
            model,
            builder,
            sender,
        });

        search_page.clone().setup_widgets();
        search_page.clone().setup_signals();
        search_page
    }

    fn setup_widgets(self: Rc<Self>) {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let row = SoukPackageRow::new(false);
            item.set_child(Some(&row));
        });

        factory.connect_bind(|_, item| {
            let child = item.get_child().unwrap();
            let row = child.clone().downcast::<SoukPackageRow>().unwrap();

            let item = item.get_item().unwrap();
            row.set_package(&item.downcast::<SoukPackage>().unwrap());
        });
        self.listview.set_factory(Some(&factory));
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::SearchEntry, search_entry);
        search_entry.connect_search_changed(clone!(@weak self as this => move|entry|{
            let text = entry.get_text().unwrap().to_string();
            if text.len() < 3 {
                return;
            }

            let packages = queries::get_packages_by_name(text, 10000, DisplayLevel::Apps).unwrap();
            this.model.remove_all();

            for package in packages{
                this.model.append(&package);
            }
        }));

        self.listview
            .connect_activate(clone!(@weak self as this => move|listview, pos|{
                let model = listview.get_model().unwrap();
                let package = model.get_object(pos).unwrap().downcast::<SoukPackage>().unwrap();
                send!(this.sender, Action::ViewSet(View::PackageDetails(package)));
            }));
    }
}
