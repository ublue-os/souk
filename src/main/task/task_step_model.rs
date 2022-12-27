// Shortwave - task_step_model.rs
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
use indexmap::set::IndexSet;

use crate::main::task::SkTaskStep;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskStepModel {
        pub map: RefCell<IndexSet<SkTaskStep>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskStepModel {
        const NAME: &'static str = "SkTaskStepModel";
        type Type = super::SkTaskStepModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkTaskStepModel {}

    impl ListModelImpl for SkTaskStepModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkTaskStep::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SkTaskStepModel(ObjectSubclass<imp::SkTaskStepModel>) @implements gio::ListModel;
}

impl SkTaskStepModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_step(&self, step: &SkTaskStep) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains(step) {
                warn!("Task step already exists in model");
                return;
            }

            map.insert(step.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_step(&self, step: &SkTaskStep) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(step) {
                Some(pos) => {
                    map.remove(step);
                    Some(pos)
                }
                None => {
                    warn!("Task step not found in model");
                    None
                }
            }
        };

        if let Some(pos) = pos {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }
    }
}

impl Default for SkTaskStepModel {
    fn default() -> Self {
        Self::new()
    }
}
