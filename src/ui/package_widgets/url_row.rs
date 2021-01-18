use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use libhandy::prelude::*;
use libhandy::subclass::prelude::*;

use std::cell::RefCell;

mod imp {
    use super::*;
    use glib::subclass;

    static PROPERTIES: [glib::subclass::Property; 2] = [
        glib::subclass::Property("url", |url| {
            glib::ParamSpec::string(
                url,
                "Url",
                "The current url for the row",
                None, // Default value
                glib::ParamFlags::READWRITE,
            )
        }),
        glib::subclass::Property("icon", |icon| {
            glib::ParamSpec::string(
                icon,
                "Icon",
                "The icon for the row",
                None,
                glib::ParamFlags::READWRITE,
            )
        }),
    ];

    #[derive(Debug, CompositeTemplate)]
    pub struct SoukUrlRow {
        #[template_child]
        pub url_type_icon: TemplateChild<gtk::Image>,

        pub url: RefCell<Option<String>>,
        pub icon: RefCell<Option<String>>,
    }

    impl ObjectSubclass for SoukUrlRow {
        const NAME: &'static str = "SoukUrlRow";
        type Type = super::SoukUrlRow;
        type ParentType = libhandy::ActionRow;
        type Class = subclass::simple::ClassStruct<Self>;
        type Instance = subclass::simple::InstanceStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                url_type_icon: TemplateChild::default(),
                url: RefCell::default(),
                icon: RefCell::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.install_properties(&PROPERTIES);
            klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/url_row.ui");
            Self::bind_template_children(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SoukUrlRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.bind_property("url", obj, "subtitle")
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build()
                .unwrap();
        }

        fn get_property(&self, _obj: &Self::Type, id: usize) -> glib::Value {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("url", ..) => self.url.borrow().to_value(),
                subclass::Property("icon", ..) => self.icon.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _obj: &Self::Type, id: usize, value: &glib::Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("url", ..) => {
                    let url: Option<String> = value.get().unwrap();
                    *self.url.borrow_mut() = url;
                }
                subclass::Property("icon", ..) => {
                    self.url_type_icon
                        .set_from_icon_name(Some(value.get().unwrap().unwrap()));
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SoukUrlRow {}

    impl ListBoxRowImpl for SoukUrlRow {}

    impl PreferencesRowImpl for SoukUrlRow {}

    impl ActionRowImpl for SoukUrlRow {
        fn activate(&self, row: &Self::Type) {
            log::debug!("Row for {} activated", row.get_title().unwrap().to_string());
            if let Some(url) = &*self.url.borrow() {
                let win = row.get_root().map(|r| r.downcast::<gtk::Window>().unwrap());
                gtk::show_uri(win.as_ref(), &url, 0);
            }
        }
    }
}

glib::wrapper! {
    pub struct SoukUrlRow(ObjectSubclass<imp::SoukUrlRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        libhandy::PreferencesRow, libhandy::ActionRow;
}
