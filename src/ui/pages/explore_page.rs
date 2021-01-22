use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libadwaita::subclass::prelude::*;

use once_cell::sync::OnceCell;

use crate::app::Action;
use crate::backend::SoukPackage;
use crate::db::{queries, DisplayLevel};
use crate::ui::utils;
use crate::ui::SoukPackageTile;
use crate::ui::View;

static EDITOR_PICKS: [&str; 7] = [
    "de.haeckerfelix.Shortwave",
    "de.haeckerfelix.Fragments",
    "org.gnome.Podcasts",
    "org.gnome.design.IconLibrary",
    "org.gnome.design.Contrast",
    "com.jetbrains.IntelliJ-IDEA-Community",
    "com.google.AndroidStudio",
];

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    pub struct SoukExplorePage {
        #[template_child]
        pub editors_picks_flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub recently_updated_flowbox: TemplateChild<gtk::FlowBox>,

        pub sender: OnceCell<Sender<Action>>,
    }

    impl ObjectSubclass for SoukExplorePage {
        const NAME: &'static str = "SoukExplorePage";
        type Type = super::SoukExplorePage;
        type ParentType = libadwaita::Bin;
        type Class = subclass::simple::ClassStruct<Self>;
        type Instance = subclass::simple::InstanceStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                editors_picks_flowbox: TemplateChild::default(),
                recently_updated_flowbox: TemplateChild::default(),
                sender: OnceCell::new(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/explore_page.ui");
            Self::bind_template_children(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SoukExplorePage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_signals();
        }
    }

    impl WidgetImpl for SoukExplorePage {}

    impl BinImpl for SoukExplorePage {}
}

glib::wrapper! {
    pub struct SoukExplorePage(ObjectSubclass<imp::SoukExplorePage>)
        @extends gtk::Widget, libadwaita::Bin;
}

impl SoukExplorePage {
    pub fn init(&self, sender: Sender<Action>) {
        let imp = imp::SoukExplorePage::from_instance(self);
        imp.sender.set(sender).unwrap();
    }

    pub fn load_data(&self) {
        let imp = imp::SoukExplorePage::from_instance(self);

        // Reset old data
        utils::clear_flowbox(&imp.editors_picks_flowbox);
        utils::clear_flowbox(&imp.recently_updated_flowbox);

        // Editors pick flowbox
        for app in &EDITOR_PICKS {
            self.add_tile(app.to_string());
        }

        // Recently updated flowbox
        for package in queries::get_recently_updated_packages(10, DisplayLevel::Apps).unwrap() {
            let tile = SoukPackageTile::new();
            tile.set_package(&package);
            imp.recently_updated_flowbox.insert(&tile, -1);
        }
    }

    fn add_tile(&self, app_id: String) {
        let imp = imp::SoukExplorePage::from_instance(self);

        if let Ok(pkg_option) =
            queries::get_package(app_id, "stable".to_string(), "flathub".to_string())
        {
            if let Some(package) = pkg_option {
                let tile = SoukPackageTile::new();
                tile.set_package(&package);
                imp.editors_picks_flowbox.insert(&tile, -1);
            }
        }
    }

    fn setup_signals(&self) {
        let imp = imp::SoukExplorePage::from_instance(self);

        let closure = clone!(@weak self as this => move |_: &gtk::FlowBox, flowbox_child: &gtk::FlowBoxChild|{
            let imp = imp::SoukExplorePage::from_instance(&this);
            let tile = flowbox_child.downcast_ref::<SoukPackageTile>().unwrap();
            let package: SoukPackage = tile.get_package().unwrap();
            send!(imp.sender.get().unwrap(), Action::ViewSet(View::PackageDetails(package)));
        });

        imp.editors_picks_flowbox
            .connect_child_activated(closure.clone());
        imp.recently_updated_flowbox
            .connect_child_activated(closure.clone());
    }
}
