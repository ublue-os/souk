// Shortwave - dry_run_runtime_model.rs
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

use crate::main::flatpak::dry_run::SkDryRunRuntime;
use crate::shared::flatpak::dry_run::DryRunRuntime;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRunRuntimeModel {
        pub map: RefCell<IndexMap<DryRunRuntime, SkDryRunRuntime>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRunRuntimeModel {
        const NAME: &'static str = "SkDryRunRuntimeModel";
        type Type = super::SkDryRunRuntimeModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkDryRunRuntimeModel {}

    impl ListModelImpl for SkDryRunRuntimeModel {
        fn item_type(&self) -> glib::Type {
            SkDryRunRuntime::static_type()
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
    pub struct SkDryRunRuntimeModel(ObjectSubclass<imp::SkDryRunRuntimeModel>) @implements gio::ListModel;
}

impl SkDryRunRuntimeModel {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_runtimes(&self, runtimes: Vec<DryRunRuntime>) {
        let imp = self.imp();

        for runtime in &runtimes {
            self.add_data(runtime);
        }

        let map = imp.map.borrow().clone();
        for data in map.keys() {
            if !runtimes.contains(data) {
                self.remove_data(data);
            }
        }
    }

    fn add_data(&self, data: &DryRunRuntime) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(data) {
                return;
            }

            let sk_runtime = SkDryRunRuntime::new(data.clone());
            map.insert(data.clone(), sk_runtime);
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    fn remove_data(&self, data: &DryRunRuntime) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(data) {
                Some(pos) => {
                    map.remove(data);
                    Some(pos)
                }
                None => {
                    warn!(
                        "Unable to remove runtime {:?}, not found in model",
                        data.package.ref_
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

impl Default for SkDryRunRuntimeModel {
    fn default() -> Self {
        Self::new()
    }
}
