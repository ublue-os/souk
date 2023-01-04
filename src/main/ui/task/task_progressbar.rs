// Souk - task_progressbar.rs
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

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{PropertyAnimationTarget, TimedAnimation};
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::glib;
use once_cell::unsync::OnceCell;

use crate::main::task::SkTask;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskProgressBar {
        pub progressbar: gtk::ProgressBar,
        pub animation: OnceCell<TimedAnimation>,

        pub task: RefCell<Option<SkTask>>,
        pub fraction: Cell<f64>,

        pub progress_watch: OnceCell<gtk::ExpressionWatch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskProgressBar {
        const NAME: &'static str = "SkTaskProgressBar";
        type ParentType = adw::Bin;
        type Type = super::SkTaskProgressBar;
    }

    impl ObjectImpl for SkTaskProgressBar {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "task",
                    "",
                    "",
                    SkTask::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "task" => obj.task().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "task" => obj.set_task(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            obj.set_child(Some(&self.progressbar));

            let target = PropertyAnimationTarget::new(&self.progressbar, "fraction");
            let animation = TimedAnimation::new(&self.progressbar, 0.0, 0.0, 1000, &target);
            self.animation.set(animation).unwrap();

            let progress_watch = obj
                .property_expression("task")
                .chain_property::<SkTask>("progress")
                .watch(
                    glib::Object::NONE,
                    clone!(@weak obj => move|| obj.update_fraction()),
                );
            self.progress_watch.set(progress_watch).unwrap();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.progress_watch.get().unwrap().unwatch();
        }
    }

    impl WidgetImpl for SkTaskProgressBar {}

    impl BinImpl for SkTaskProgressBar {}
}

glib::wrapper! {
    pub struct SkTaskProgressBar(
        ObjectSubclass<imp::SkTaskProgressBar>)
        @extends gtk::Widget, adw::Bin;
}

impl SkTaskProgressBar {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn task(&self) -> Option<SkTask> {
        self.imp().task.borrow().clone()
    }

    pub fn set_task(&self, task: &SkTask) {
        *self.imp().task.borrow_mut() = Some(task.clone());

        self.notify("task");
    }

    pub fn update_fraction(&self) {
        let animation = self.imp().animation.get().unwrap();
        let current_value = animation.value();

        if let Some(task) = self.task() {
            animation.reset();
            animation.set_value_from(current_value);
            animation.set_value_to(task.progress() as f64);
            animation.play();
        }
    }
}

impl Default for SkTaskProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
