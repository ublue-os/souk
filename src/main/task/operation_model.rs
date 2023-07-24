// Souk - operation_model.rs
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

use super::SkOperation;
use crate::shared::task::response::OperationActivity;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkOperationModel {
        pub map: RefCell<IndexMap<String, SkOperation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkOperationModel {
        const NAME: &'static str = "SkOperationModel";
        type Type = super::SkOperationModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkOperationModel {}

    impl ListModelImpl for SkOperationModel {
        fn item_type(&self) -> glib::Type {
            SkOperation::static_type()
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
    pub struct SkOperationModel(ObjectSubclass<imp::SkOperationModel>) @implements gio::ListModel;
}

impl SkOperationModel {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub(super) fn handle_activity(&self, activity: &Vec<OperationActivity>) {
        for oa in activity {
            if let Some(operation) = self.operation(&oa.identifier()) {
                operation.handle_activity(oa);
            } else {
                let op = SkOperation::new(self.n_items(), oa);
                self.add_operation(&op);
            }
        }
    }

    pub(super) fn operation(&self, identifier: &str) -> Option<SkOperation> {
        self.imp().map.borrow().get(identifier).cloned()
    }

    fn add_operation(&self, operation: &SkOperation) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&operation.identifier()) {
                warn!(
                    "Operation {:?} already exists in model",
                    operation.identifier()
                );
                return;
            }

            map.insert(operation.identifier(), operation.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }
}

impl Default for SkOperationModel {
    fn default() -> Self {
        Self::new()
    }
}
