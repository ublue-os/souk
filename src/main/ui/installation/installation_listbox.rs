// Souk - installation_listbox.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::{clone, ParamSpec, Properties};
use gtk::glib;

use crate::main::app::SkApplication;
use crate::main::flatpak::installation::SkInstallation;
use crate::main::ui::installation::SkInstallationRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkInstallationListBox)]
    pub struct SkInstallationListBox {
        #[property(get, set = Self::set_selected_installation)]
        pub selected_installation: RefCell<Option<SkInstallation>>,

        pub listbox: gtk::ListBox,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationListBox {
        const NAME: &'static str = "SkInstallationListBox";
        type ParentType = adw::Bin;
        type Type = super::SkInstallationListBox;
    }

    impl ObjectImpl for SkInstallationListBox {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_child(Some(&self.obj().imp().listbox));

            self.listbox.set_selection_mode(gtk::SelectionMode::None);
            self.listbox.add_css_class("boxed-list");

            let worker = SkApplication::default().worker();

            self.listbox
                .connect_row_activated(clone!(@weak self as this => move |_, row|{
                    let row = row.downcast_ref::<SkInstallationRow>().unwrap();

                    this.unselect_all();
                    row.set_selected(true);

                    *this.selected_installation.borrow_mut() = Some(row.installation());
                    this.obj().notify("selected-installation");
                }));

            self.listbox
                .bind_model(Some(&worker.installations()), |installation| {
                    let installation = installation.downcast_ref::<SkInstallation>().unwrap();
                    SkInstallationRow::new(installation).upcast()
                });

            worker.installations().connect_items_changed(
                clone!(@weak self as this => move |_, _, _, _|{
                    if let Some(selected) = this.obj().selected_installation(){
                        let mut index = 0;
                        while let Some(row) = this.listbox.row_at_index(index) {
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
    }

    impl WidgetImpl for SkInstallationListBox {}

    impl BinImpl for SkInstallationListBox {}

    impl SkInstallationListBox {
        fn set_selected_installation(&self, selected_installation: &SkInstallation) {
            let mut index = 0;
            while let Some(row) = self.listbox.row_at_index(index) {
                let row = row.downcast_ref::<SkInstallationRow>().unwrap();
                if &row.installation() == selected_installation {
                    row.set_selected(true);

                    *self.selected_installation.borrow_mut() = Some(row.installation());
                    self.obj().notify("selected-installation");
                    return;
                }

                index += 1;
            }
        }

        fn unselect_all(&self) {
            let mut index = 0;
            while let Some(row) = self.listbox.row_at_index(index) {
                let row = row.downcast_ref::<SkInstallationRow>().unwrap();
                row.set_selected(false);

                index += 1;
            }
        }
    }
}

glib::wrapper! {
    pub struct SkInstallationListBox(
        ObjectSubclass<imp::SkInstallationListBox>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for SkInstallationListBox {
    fn default() -> Self {
        glib::Object::new()
    }
}
