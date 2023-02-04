// Souk - task_model.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use super::{SkTask, SkTaskStatus};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskModel {
        pub map: RefCell<IndexMap<String, SkTask>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskModel {
        const NAME: &'static str = "SkTaskModel";
        type Type = super::SkTaskModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkTaskModel {}

    impl ListModelImpl for SkTaskModel {
        fn item_type(&self) -> glib::Type {
            SkTask::static_type()
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
    pub struct SkTaskModel(ObjectSubclass<imp::SkTaskModel>) @implements gio::ListModel;
}

impl SkTaskModel {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn add_task(&self, task: &SkTask) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&task.uuid()) {
                warn!("Task {:?} already exists in model", task.uuid());
                return;
            }

            map.insert(task.uuid(), task.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_task(&self, task: &SkTask) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(&task.uuid()) {
                Some(pos) => {
                    map.shift_remove(&task.uuid());
                    Some(pos)
                }
                None => {
                    warn!("Task {:?} not found in model", task.uuid());
                    None
                }
            }
        };

        if let Some(pos) = pos {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }
    }

    pub fn task(&self, uuid: &str) -> Option<SkTask> {
        self.imp().map.borrow().get(uuid).cloned()
    }

    pub fn remove_completed_tasks(&self, keep: u32) {
        let completed_expression =
            gtk::PropertyExpression::new(SkTask::static_type(), gtk::Expression::NONE, "status")
                .chain_closure::<bool>(closure!(|_: Option<glib::Object>, status: SkTaskStatus| {
                status.is_completed()
            }));

        let filter = gtk::BoolFilter::new(Some(&completed_expression));
        let filtermodel = gtk::FilterListModel::new(Some(self), Some(&filter));

        while filtermodel.n_items() >= keep {
            let task = filtermodel.item(0).unwrap().downcast::<SkTask>().unwrap();
            self.remove_task(&task);
        }
    }
}

impl Default for SkTaskModel {
    fn default() -> Self {
        Self::new()
    }
}
