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

use std::cell::Cell;
use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{PropertyAnimationTarget, TimedAnimation};
use glib::{ParamFlags, ParamSpec, ParamSpecDouble, ParamSpecObject};
use gtk::glib;
use gtk::subclass::prelude::*;

use crate::main::task::SkTask;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskProgressBar {
        pub progressbar: gtk::ProgressBar,
        pub animation: TimedAnimation,

        pub task: RefCell<Option<SkTask>>,
        pub fraction: Cell<f64>,
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
                vec![
                    ParamSpecObject::new(
                        "task",
                        "",
                        "",
                        SkTask::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecDouble::new(
                        "fraction",
                        "",
                        "",
                        0.0,
                        f64::MAX,
                        0.0,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "task" => obj.task().to_value(),
                "fraction" => obj.fraction().to_value(),
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
                "fraction" => obj.set_fraction(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            obj.set_child(Some(&self.progressbar));

            //let animation = TimedAnimation::new(&self.progressbar, 0, 0, 500, target)
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
        self.bind_properties();
    }

    pub fn fraction(&self) -> f64 {
        self.imp().fraction.get()
    }

    pub fn set_fraction(&self, fraction: f64) {
        self.imp().fraction.set(fraction);
        self.notify("fraction");
    }

    fn bind_properties(&self) {
        let progressbar = &self.imp().progressbar;
        let task = self.task().unwrap();

        task.bind_property("progress", progressbar, "fraction")
            .build();
    }
}

