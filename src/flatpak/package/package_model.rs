// Shortwave - package_model.rs
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
use std::convert::TryInto;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::flatpak::package::SkPackage;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackageModel {
        pub map: RefCell<IndexMap<String, SkPackage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackageModel {
        const NAME: &'static str = "SkPackageModel";
        type Type = super::SkPackageModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkPackageModel {}

    impl ListModelImpl for SkPackageModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkPackage::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SkPackageModel(ObjectSubclass<imp::SkPackageModel>) @implements gio::ListModel;
}

impl SkPackageModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_package(&self, package: &SkPackage) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&package.id()) {
                warn!("Package {:?} already exists in model", package.id());
                return;
            }

            map.insert(package.id(), package.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_package(&self, package: &SkPackage) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(&package.id()) {
                Some(pos) => {
                    map.remove(&package.id());
                    Some(pos)
                }
                None => {
                    warn!("Package {:?} not found in model", package.id());
                    None
                }
            }
        };

        if let Some(pos) = pos {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }
    }

    pub fn package(&self, id: &str) -> Option<SkPackage> {
        self.imp().map.borrow().get(id).cloned()
    }
}

impl Default for SkPackageModel {
    fn default() -> Self {
        Self::new()
    }
}
