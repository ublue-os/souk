// Shortwave - package_model.rs
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
use std::convert::TryInto;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::main::flatpak::package::SkPackage;
use crate::shared::flatpak::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackageModel {
        pub map: RefCell<IndexMap<PackageInfo, SkPackage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackageModel {
        const NAME: &'static str = "SkPackageModel";
        type Type = super::SkPackageModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkPackageModel {}

    impl ListModelImpl for SkPackageModel {
        fn item_type(&self) -> glib::Type {
            SkPackage::static_type()
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
}

glib::wrapper! {
    pub struct SkPackageModel(ObjectSubclass<imp::SkPackageModel>) @implements gio::ListModel;
}

impl SkPackageModel {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_packages(&self, packages: Vec<PackageInfo>) {
        let imp = self.imp();

        for package in &packages {
            self.add_info(package);
        }

        let map = imp.map.borrow().clone();
        for info in map.keys() {
            if !packages.contains(info) {
                self.remove_info(info);
            }
        }
    }

    fn add_info(&self, info: &PackageInfo) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(info) {
                return;
            }

            let sk_package = SkPackage::new(info);
            map.insert(info.clone(), sk_package);
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    fn remove_info(&self, info: &PackageInfo) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(info) {
                Some(pos) => {
                    map.remove(info);
                    Some(pos)
                }
                None => {
                    warn!(
                        "Unable to remove package {:?}, not found in model",
                        info.ref_
                    );
                    None
                }
            }
        };

        if let Some(pos) = pos {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }
    }
}

impl Default for SkPackageModel {
    fn default() -> Self {
        Self::new()
    }
}
