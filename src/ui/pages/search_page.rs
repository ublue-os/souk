use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use once_cell::sync::OnceCell;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::db::{queries, DisplayLevel};
use crate::ui::SoukPackageRow;
use crate::ui::View;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    pub struct SoukSearchPage {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        pub model: gio::ListStore,
        pub sender: OnceCell<Sender<Action>>,
    }

    impl ObjectSubclass for SoukSearchPage {
        const NAME: &'static str = "SoukSearchPage";
        type Type = super::SoukSearchPage;
        type ParentType = gtk::Box;
        type Class = subclass::simple::ClassStruct<Self>;
        type Instance = subclass::simple::InstanceStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                listview: TemplateChild::default(),
                search_entry: TemplateChild::default(),
                model: gio::ListStore::new(SoukPackage::static_type()),
                sender: OnceCell::new(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/search_page.ui");
            Self::bind_template_children(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SoukSearchPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let selection_model = gtk::NoSelection::new(Some(&self.model));
            self.listview.set_model(Some(&selection_model));

            obj.setup_widgets();
            obj.setup_signals();
        }
    }
    impl WidgetImpl for SoukSearchPage {}
    impl BoxImpl for SoukSearchPage {}
}

glib::wrapper! {
    pub struct SoukSearchPage(ObjectSubclass<imp::SoukSearchPage>)
        @extends gtk::Widget, gtk::Box;
}

impl SoukSearchPage {
    pub fn init(&self, sender: Sender<Action>) {
        let imp = imp::SoukSearchPage::from_instance(self);

        let _ = imp.sender.set(sender);
    }

    pub fn setup_widgets(&self) {
        let imp = imp::SoukSearchPage::from_instance(self);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let row = SoukPackageRow::new(false);
            item.set_child(Some(&row));
        });

        factory.connect_bind(|_, item| {
            let child = item.get_child().unwrap();
            let row = child.downcast_ref::<SoukPackageRow>().unwrap();

            let item = item.get_item().unwrap();
            row.set_package(&item.downcast::<SoukPackage>().unwrap());
        });
        imp.listview.set_factory(Some(&factory));
    }

    pub fn setup_signals(&self) {
        let imp = imp::SoukSearchPage::from_instance(self);

        imp.search_entry.connect_search_changed(clone!(@weak self as this => move|entry|{
            let imp = imp::SoukSearchPage::from_instance(&this);

            let text = entry.get_text().unwrap().to_string();
            if text.len() < 3 {
                return;
            }

            let packages = queries::get_packages_by_name(text, 10000, DisplayLevel::Apps).unwrap();
            imp.model.remove_all();

            for package in packages{
                imp.model.append(&package);
            }
        }));

        imp.listview
            .connect_activate(clone!(@weak self as this => move|listview, pos|{
                let imp = imp::SoukSearchPage::from_instance(&this);

                let model = listview.get_model().unwrap();
                let package = model.get_object(pos).unwrap().downcast::<SoukPackage>().unwrap();
                send!(imp.sender.get().unwrap(), Action::ViewSet(View::PackageDetails(package)));
            }));
    }
}
