// Souk - dry_run_package_model.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
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
use std::convert::TryInto;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::main::flatpak::dry_run::SkDryRunPackage;
use crate::shared::flatpak::dry_run::DryRunPackage;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRunPackageModel {
        pub map: RefCell<IndexMap<DryRunPackage, SkDryRunPackage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRunPackageModel {
        const NAME: &'static str = "SkDryRunPackageModel";
        type Type = super::SkDryRunPackageModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkDryRunPackageModel {}

    impl ListModelImpl for SkDryRunPackageModel {
        fn item_type(&self) -> glib::Type {
            SkDryRunPackage::static_type()
        }

        fn n_items(&self) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }

    impl SkDryRunPackageModel {
        pub fn add_data(&self, data: &DryRunPackage) {
            let pos = {
                let mut map = self.map.borrow_mut();
                if map.contains_key(data) {
                    return;
                }

                let sk_package = SkDryRunPackage::new(data.clone());
                map.insert(data.clone(), sk_package);
                (map.len() - 1) as u32
            };

            self.obj().items_changed(pos, 0, 1);
        }

        pub fn remove_data(&self, data: &DryRunPackage) {
            let pos = {
                let mut map = self.map.borrow_mut();
                match map.get_index_of(data) {
                    Some(pos) => {
                        map.remove(data);
                        Some(pos)
                    }
                    None => {
                        warn!(
                            "Unable to remove package {:?}, not found in model",
                            data.info.ref_
                        );
                        None
                    }
                }
            };

            if let Some(pos) = pos {
                self.obj().items_changed(pos.try_into().unwrap(), 1, 0);
            }
        }
    }
}

glib::wrapper! {
    pub struct SkDryRunPackageModel(ObjectSubclass<imp::SkDryRunPackageModel>) @implements gio::ListModel;
}

impl SkDryRunPackageModel {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_packages(&self, packages: Vec<DryRunPackage>) {
        let imp = self.imp();

        for package in &packages {
            imp.add_data(package);
        }

        let map = imp.map.borrow().clone();
        for data in map.keys() {
            if !packages.contains(data) {
                imp.remove_data(data);
            }
        }
    }
}

impl Default for SkDryRunPackageModel {
    fn default() -> Self {
        Self::new()
    }
}
