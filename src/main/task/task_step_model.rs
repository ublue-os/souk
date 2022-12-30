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

use glib::{ParamFlags, ParamSpec, ParamSpecObject, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::set::IndexSet;
use once_cell::sync::Lazy;

use crate::main::task::SkTaskStep;
use crate::shared::task::TaskStep;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskStepModel {
        pub steps: RefCell<IndexSet<SkTaskStep>>,
        pub current: RefCell<Option<SkTaskStep>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskStepModel {
        const NAME: &'static str = "SkTaskStepModel";
        type Type = super::SkTaskStepModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkTaskStepModel {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "current",
                    "",
                    "",
                    SkTaskStep::static_type(),
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "current" => obj.current().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for SkTaskStepModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkTaskStep::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.steps.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.steps
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

    /// Returns the [SkTaskStep] that was updated last
    pub fn current(&self) -> Option<SkTaskStep> {
        self.imp().current.borrow().clone()
    }

    pub(super) fn set_steps(&self, steps: &Vec<TaskStep>) {
        for step in steps {
            let task_step = SkTaskStep::new(step);
            self.add_step(&task_step);
        }
    }

    /// Updates the data of a SkStep, and sets it as `current` step
    pub(super) fn update_step(&self, updated: Option<&TaskStep>) {
        if let Some(updated) = updated {
            let task_step: SkTaskStep = self.item(updated.index).unwrap().downcast().unwrap();
            task_step.update(updated);

            *self.imp().current.borrow_mut() = Some(task_step);
        } else {
            *self.imp().current.borrow_mut() = None;
        }

        self.notify("current");
    }

    fn add_step(&self, step: &SkTaskStep) {
        let pos = {
            let mut steps = self.imp().steps.borrow_mut();
            if steps.contains(step) {
                warn!("Task step already exists in model");
                return;
            }

            steps.insert(step.clone());
            (steps.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }
}

impl Default for SkTaskStepModel {
    fn default() -> Self {
        Self::new()
    }
}
