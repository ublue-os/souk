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
    selection_model: gtk::NoSelection,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl SearchPage {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/search_page.ui");
        get_widget!(builder, gtk::Box, search_page);
        get_widget!(builder, gtk::ListView, listview);

        // TODO: Currently we're adding a GtkListView to a HdyClamp
        // This doesn't make any sense and defeats the purpose of using a GtkListView
        // since with using a HdyClamp all rows are getting rendered.
        // As soon as we have a helper for this in libhandy/libadwaita we have to update
        // it here (e.g. HdyScrolledClamp)

        let model = gio::ListStore::new(SoukPackage::static_type());
        let selection_model = gtk::NoSelection::new(Some(&model));
        listview.set_model(Some(&selection_model));

        let search_page = Rc::new(Self {
            widget: search_page,
            listview,
            model,
            selection_model,
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
            let row = SoukPackageRow::new();
            item.set_child(Some(&row));
        });

        factory.connect_bind(|_, item| {
            let child = item.get_child().unwrap();
            let row = child.clone().downcast::<SoukPackageRow>().unwrap();

            let item = item.get_item().unwrap();
            row.set_property("package", &item).unwrap();
        });
        self.listview.set_factory(Some(&factory));
    }

    fn setup_signals(self: Rc<Self>) {
        get_widget!(self.builder, gtk::SearchEntry, search_entry);
        search_entry.connect_search_changed(clone!(@weak self as this => move|entry|{
            let text = entry.get_text().unwrap().to_string();
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
