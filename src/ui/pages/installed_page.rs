use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use once_cell::sync::OnceCell;

use crate::app::Action;
use crate::backend::{SoukFlatpakBackend, SoukPackage, SoukPackageKind};
use crate::ui::SoukPackageRow;
use crate::ui::View;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    pub struct SoukInstalledPage {
        #[template_child]
        pub listbox_apps: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub listbox_runtimes: TemplateChild<gtk::ListBox>,

        pub sender: OnceCell<Sender<Action>>,
        pub flatpak_backend: OnceCell<SoukFlatpakBackend>,
    }

    impl ObjectSubclass for SoukInstalledPage {
        const NAME: &'static str = "SoukInstalledPage";
        type Type = super::SoukInstalledPage;
        type ParentType = gtk::Widget;
        type Class = subclass::simple::ClassStruct<Self>;
        type Instance = subclass::simple::InstanceStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                listbox_apps: TemplateChild::default(),
                listbox_runtimes: TemplateChild::default(),
                sender: OnceCell::new(),
                flatpak_backend: OnceCell::new(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/installed_page.ui");
            Self::bind_template_children(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SoukInstalledPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_signals();
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.get_first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SoukInstalledPage {}
}

glib::wrapper! {
    pub struct SoukInstalledPage(ObjectSubclass<imp::SoukInstalledPage>) @extends gtk::Widget;
}

impl SoukInstalledPage {
    pub fn init(&self, sender: Sender<Action>, flatpak_backend: SoukFlatpakBackend) {
        let imp = imp::SoukInstalledPage::from_instance(self);

        imp.sender.set(sender).unwrap();
        imp.flatpak_backend.set(flatpak_backend).unwrap();
        self.setup_widgets();
    }

    fn setup_widgets(&self) {
        let imp = imp::SoukInstalledPage::from_instance(self);

        let model: gio::ListStore = imp.flatpak_backend.get().unwrap().get_installed_packages();

        // Apps section
        let apps_filter = gtk::CustomFilter::new(|object| {
            let package = object.clone().downcast::<SoukPackage>().unwrap();
            package.get_kind() == SoukPackageKind::App
        });
        let apps_model = gtk::FilterListModel::new(Some(&model), Some(&apps_filter));

        imp.listbox_apps.bind_model(Some(&apps_model), |package| {
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

        imp.listbox_runtimes
            .bind_model(Some(&runtimes_model), |package| {
                let row = SoukPackageRow::new(true);
                row.set_package(&package.clone().downcast::<SoukPackage>().unwrap());
                row.upcast::<gtk::Widget>()
            });
    }

    fn setup_signals(&self) {
        let imp = imp::SoukInstalledPage::from_instance(self);

        let closure = clone!(@weak self as this => move|_: &gtk::ListBox, listbox_row: &gtk::ListBoxRow|{
            let imp = imp::SoukInstalledPage::from_instance(&this);
            let row = listbox_row.clone().downcast::<SoukPackageRow>().unwrap();
            let package: SoukPackage = row.get_package().unwrap();
            send!(imp.sender.get().unwrap(), Action::ViewSet(View::PackageDetails(package)));
        });

        imp.listbox_apps.connect_row_activated(closure.clone());
        imp.listbox_runtimes.connect_row_activated(closure.clone());
    }
}
