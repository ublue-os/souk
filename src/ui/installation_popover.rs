// Souk - installation_popover.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;

use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use libflatpak::Installation;
use once_cell::sync::Lazy;

use crate::app::SkApplication;
use crate::ui::SkInstallationRow;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/installation_popover.ui")]
    pub struct SkInstallationPopover {
        #[template_child]
        pub listbox: TemplateChild<gtk::ListBox>,

        pub selected_installation: RefCell<Option<Installation>>,
        pub selected_installation_title: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationPopover {
        const NAME: &'static str = "SkInstallationPopover";
        type ParentType = gtk::Popover;
        type Type = super::SkInstallationPopover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkInstallationPopover {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "selected-installation",
                        "Selected Installation",
                        "Selected Installation",
                        Installation::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "selected-installation-title",
                        "Selected Installation Title",
                        "Selected Installation Title",
                        None,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-installation" => obj.selected_installation().to_value(),
                "selected-installation-title" => obj.selected_installation_title().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_signals();
        }
    }

    impl WidgetImpl for SkInstallationPopover {}

    impl PopoverImpl for SkInstallationPopover {}
}

glib::wrapper! {
    pub struct SkInstallationPopover(
        ObjectSubclass<imp::SkInstallationPopover>)
        @extends gtk::Widget, gtk::Popover;
}

impl SkInstallationPopover {
    fn setup_signals(&self) {
        let imp = self.imp();
        let worker = SkApplication::default().worker();

        imp.listbox
            .connect_row_activated(clone!(@weak self as this => move |_, row|{
                let row = row.downcast_ref::<SkInstallationRow>().unwrap();

                this.unselect_all();
                row.set_selected(true);

                *this.imp().selected_installation.borrow_mut() = Some(row.installation());
                *this.imp().selected_installation_title.borrow_mut() = row.installation_title();
                this.notify("selected-installation");
                this.notify("selected-installation-title");

                this.hide();
            }));

        imp.listbox
            .bind_model(Some(&worker.installations()), |installation| {
                SkInstallationRow::new(installation.downcast_ref::<Installation>().unwrap())
                    .upcast()
            });
    }

    pub fn set_installation(&self, installation: &Installation) {
        let mut index = 0;
        while let Some(row) = self.imp().listbox.row_at_index(index) {
            let row = row.downcast_ref::<SkInstallationRow>().unwrap();
            if &row.installation() == installation {
                row.set_selected(true);
                return;
            }

            index += 1;
        }
    }

    pub fn selected_installation(&self) -> Option<Installation> {
        self.imp().selected_installation.borrow().clone()
    }

    pub fn selected_installation_title(&self) -> String {
        self.imp().selected_installation_title.borrow().clone()
    }

    fn unselect_all(&self) {
        let mut index = 0;
        while let Some(row) = self.imp().listbox.row_at_index(index) {
            let row = row.downcast_ref::<SkInstallationRow>().unwrap();
            row.set_selected(false);

            index += 1;
        }
    }
}

impl Default for SkInstallationPopover {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}
