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
use std::rc::Rc;
use std::time::Duration;

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
        pub is_pulsing: Rc<Cell<bool>>,

        pub task: RefCell<Option<SkTask>>,
        pub fraction: Cell<f64>,

        pub watches: RefCell<Vec<gtk::ExpressionWatch>>,
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

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "task" => self.obj().task().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "task" => self.obj().set_task(value.get().ok()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.progressbar.set_pulse_step(1.0);
            self.progressbar.set_valign(gtk::Align::Center);
            self.obj().set_child(Some(&self.progressbar));

            let target = PropertyAnimationTarget::new(&self.progressbar, "fraction");
            let animation = TimedAnimation::new(&self.progressbar, 0.0, 0.0, 1000, &target);
            self.animation.set(animation).unwrap();
        }

        fn dispose(&self) {
            // Workaround copied from
            // https://github.com/YaLTeR/plitki/blob/b0c43452e407d906c57b55fdb08980aed29831e4/plitki-gnome/src/hit_light.rs#L49
            let animation = self.animation.get().unwrap();
            animation.set_target(&adw::CallbackAnimationTarget::new(|_| ()));
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
        glib::Object::new(&[])
    }

    pub fn task(&self) -> Option<SkTask> {
        self.imp().task.borrow().clone()
    }

    pub fn set_task(&self, task: Option<&SkTask>) {
        let imp = self.imp();

        while let Some(watch) = imp.watches.borrow_mut().pop() {
            watch.unwatch();
        }

        if let Some(task) = task {
            let watch = task.property_expression("progress").watch(
                glib::Object::NONE,
                clone!(@weak self as this => move|| this.update_fraction()),
            );
            imp.watches.borrow_mut().push(watch);

            let watch = task.property_expression("status").watch(
                glib::Object::NONE,
                clone!(@weak self as this => move|| this.update()),
            );
            imp.watches.borrow_mut().push(watch);
        }

        *imp.task.borrow_mut() = task.cloned();
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

    pub fn update(&self) {
        if let Some(task) = self.task() {
            let is_pulsing = self.imp().is_pulsing.get();

            // Show a pulse animation, since we don't have any progress information
            // available when a Flatpak bundle gets installed.
            if task.status().has_no_detailed_progress() && !is_pulsing {
                self.imp().is_pulsing.set(true);
                glib::timeout_add_local(
                    Duration::from_secs(1),
                    clone!(@weak self as this => @default-return Continue(false), move || {
                        let is_pulsing = this.imp().is_pulsing.get();

                        if is_pulsing {
                            this.imp().progressbar.pulse();
                        } else {
                            this.update_fraction();
                        }

                        Continue(is_pulsing)
                    }),
                );
            } else {
                self.imp().is_pulsing.set(false);
            }
        }
    }
}

impl Default for SkTaskProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
