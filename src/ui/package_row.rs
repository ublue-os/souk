use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};

use std::cell::Cell;
use std::cell::RefCell;

use crate::backend::SoukPackage;
use crate::ui::utils;

#[derive(Debug, CompositeTemplate)]
pub struct SoukPackageRowPrivate {
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub summary_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub icon_image: TemplateChild<gtk::Image>,
    #[template_child]
    pub branch_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub installed_check: TemplateChild<gtk::Image>,
    #[template_child]
    pub uninstall_box: TemplateChild<gtk::Box>,
    #[template_child]
    pub uninstall_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub installed_size_label: TemplateChild<gtk::Label>,

    package: RefCell<Option<SoukPackage>>,
    installed_view: Cell<bool>,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property("package", |package| {
    glib::ParamSpec::object(
        package,
        "Package",
        "Package",
        SoukPackage::static_type(),
        glib::ParamFlags::READWRITE,
    )
})];

impl ObjectSubclass for SoukPackageRowPrivate {
    const NAME: &'static str = "SoukPackageRow";
    type Type = SoukPackageRow;
    type ParentType = gtk::ListBoxRow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
        klass.set_template_from_resource("/de/haeckerfelix/Souk/gtk/package_row.ui");
        Self::bind_template_children(klass);
    }

    glib::object_subclass!();

    fn new() -> Self {
        let package = RefCell::new(None);
        let installed_view = Cell::default();

        Self {
            title_label: TemplateChild::default(),
            summary_label: TemplateChild::default(),
            icon_image: TemplateChild::default(),
            branch_label: TemplateChild::default(),
            installed_check: TemplateChild::default(),
            uninstall_box: TemplateChild::default(),
            uninstall_button: TemplateChild::default(),
            installed_size_label: TemplateChild::default(),
            package,
            installed_view,
        }
    }

    fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
        obj.init_template();
    }
}

impl ObjectImpl for SoukPackageRowPrivate {
    fn set_property(&self, _obj: &SoukPackageRow, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => {
                let package = value.get().unwrap();
                *self.package.borrow_mut() = package;
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &SoukPackageRow, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => self.package.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for SoukPackageRowPrivate {}

impl ListBoxRowImpl for SoukPackageRowPrivate {}

glib::wrapper! {
    pub struct SoukPackageRow(ObjectSubclass<SoukPackageRowPrivate>)
    @extends gtk::Widget, gtk::ListBoxRow;
}

impl SoukPackageRow {
    pub fn new(installed_view: bool) -> Self {
        let row = glib::Object::new::<Self>(&[]).unwrap();

        let self_ = SoukPackageRowPrivate::from_instance(&row);
        self_.installed_view.set(installed_view);

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        self.connect_notify(Some("package"), |this, _| {
            let self_ = SoukPackageRowPrivate::from_instance(this);
            let package = self_.package.borrow().as_ref().unwrap().clone();

            // Icon
            utils::set_icon(&package, &self_.icon_image, 64);

            match package.get_appdata() {
                Some(appdata) => {
                    // Title
                    utils::set_label_translatable_string(
                        &self_.title_label,
                        Some(appdata.name.clone()),
                    );
                    // Summary
                    utils::set_label_translatable_string(
                        &self_.summary_label,
                        appdata.summary.clone(),
                    );
                }
                None => {
                    // Fallback to basic information when no appdata available
                    self_.title_label.set_text(&package.get_name());
                    self_.summary_label.set_text(&package.get_branch());
                }
            };

            // Installed indicator
            if !self_.installed_view.get() {
                package
                    .bind_property("is_installed", &*self_.installed_check, "visible")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build()
                    .unwrap();
            }

            // Branch label / tag
            let branch = package.get_branch();
            if branch != "stable" {
                self_.branch_label.set_text(&branch.to_uppercase());
                self_.branch_label.set_visible(true);

                let ctx = self_.branch_label.get_style_context();
                ctx.remove_class("branch-label-orange");
                ctx.remove_class("branch-label-red");

                if branch == "beta" {
                    ctx.add_class("branch-label-orange");
                }

                if branch == "master" {
                    ctx.add_class("branch-label-red");
                }
            } else {
                self_.branch_label.set_visible(false);
            }

            // Uninstall button
            self_.uninstall_button.set_sensitive(true);
            if self_.installed_view.get() {
                self_.uninstall_box.set_visible(true);

                let bytes = package
                    .get_installed_info()
                    .as_ref()
                    .unwrap()
                    .get_installed_size();
                let size = glib::format_size(bytes);
                self_.installed_size_label.set_text(&size);

                self_
                    .uninstall_button
                    .connect_clicked(clone!(@weak package => move|btn|{
                        btn.set_sensitive(false);
                        package.uninstall();
                    }));
            }
        });
    }

    pub fn set_package(&self, package: &SoukPackage) {
        self.set_property("package", package).unwrap();
    }

    pub fn get_package(&self) -> Option<SoukPackage> {
        self.get_property("package").unwrap().get().unwrap()
    }
}
