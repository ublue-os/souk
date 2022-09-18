// Souk - installation_listbox.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

use crate::app::SkApplication;
use crate::flatpak::installation::SkInstallation;
use crate::ui::installation::SkInstallationRow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkInstallationListBox {
        pub listbox: gtk::ListBox,
        pub selected_installation: RefCell<Option<SkInstallation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationListBox {
        const NAME: &'static str = "SkInstallationListBox";
        type ParentType = adw::Bin;
        type Type = super::SkInstallationListBox;
    }

    impl ObjectImpl for SkInstallationListBox {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "selected-installation",
                    "",
                    "",
                    SkInstallation::static_type(),
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-installation" => obj.selected_installation().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.set_child(Some(&obj.imp().listbox));

            self.listbox.set_selection_mode(gtk::SelectionMode::None);
            self.listbox.add_css_class("boxed-list");

            obj.setup_signals();
        }
    }

    impl WidgetImpl for SkInstallationListBox {}

    impl BinImpl for SkInstallationListBox {}
}

glib::wrapper! {
    pub struct SkInstallationListBox(
        ObjectSubclass<imp::SkInstallationListBox>)
        @extends gtk::Widget, adw::Bin;
}

impl SkInstallationListBox {
    fn setup_signals(&self) {
        let imp = self.imp();
        let worker = SkApplication::default().worker();

        imp.listbox
            .connect_row_activated(clone!(@weak self as this => move |_, row|{
                let row = row.downcast_ref::<SkInstallationRow>().unwrap();

                this.unselect_all();
                row.set_selected(true);

                *this.imp().selected_installation.borrow_mut() = Some(row.installation());
                this.notify("selected-installation");
            }));

        imp.listbox
            .bind_model(Some(&worker.installations()), |installation| {
                let installation = installation.downcast_ref::<SkInstallation>().unwrap();
                SkInstallationRow::new(installation).upcast()
            });

        worker.installations().connect_items_changed(
            clone!(@weak self as this => move |_, _, _, _|{
                if let Some(selected) = this.selected_installation(){
                    let mut index = 0;
                    while let Some(row) = this.imp().listbox.row_at_index(index) {
                        let row = row.downcast_ref::<SkInstallationRow>().unwrap();
                        if row.installation() == selected{
                            row.set_selected(true);
                            return;
                        }

                        index += 1;
                    }
                }
            }),
        );
    }

    pub fn set_selected_installation(&self, selected_installation: &SkInstallation) {
        let mut index = 0;
        while let Some(row) = self.imp().listbox.row_at_index(index) {
            let row = row.downcast_ref::<SkInstallationRow>().unwrap();
            if &row.installation() == selected_installation {
                row.set_selected(true);

                *self.imp().selected_installation.borrow_mut() = Some(row.installation());
                self.notify("selected-installation");
                return;
            }

            index += 1;
        }
    }

    pub fn selected_installation(&self) -> Option<SkInstallation> {
        self.imp().selected_installation.borrow().clone()
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

impl Default for SkInstallationListBox {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}
