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
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{PropertyAnimationTarget, TimedAnimation};
use glib::{clone, ParamSpec, Properties};
use gtk::glib;
use once_cell::unsync::OnceCell;

use crate::main::task::SkTask;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkTaskProgressBar)]
    pub struct SkTaskProgressBar {
        #[property(get, set = Self::set_task)]
        task: RefCell<Option<SkTask>>,

        progressbar: gtk::ProgressBar,
        animation: OnceCell<TimedAnimation>,
        is_pulsing: Cell<bool>,

        watches: RefCell<Vec<gtk::ExpressionWatch>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskProgressBar {
        const NAME: &'static str = "SkTaskProgressBar";
        type ParentType = adw::Bin;
        type Type = super::SkTaskProgressBar;
    }

    impl ObjectImpl for SkTaskProgressBar {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.progressbar.set_pulse_step(1.0);
            self.progressbar.set_valign(gtk::Align::Center);
            self.obj().set_child(Some(&self.progressbar));

            let target = PropertyAnimationTarget::new(&self.progressbar, "fraction");
            let animation = TimedAnimation::new(&self.progressbar, 0.0, 0.0, 1000, target);
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

    impl SkTaskProgressBar {
        fn set_task(&self, task: Option<&SkTask>) {
            while let Some(watch) = self.watches.borrow_mut().pop() {
                watch.unwatch();
            }

            if let Some(task) = task {
                let watch = task.property_expression("progress").watch(
                    glib::Object::NONE,
                    clone!(@weak self as this => move|| this.update_fraction()),
                );
                self.watches.borrow_mut().push(watch);

                let watch = task.property_expression("current-operation").watch(
                    glib::Object::NONE,
                    clone!(@weak self as this => move|| this.update()),
                );
                self.watches.borrow_mut().push(watch);
            }

            *self.task.borrow_mut() = task.cloned();
            self.obj().notify("task");

            self.update_fraction();
        }

        fn update(&self) {
            let no_detailed_progress = if let Some(task) = self.obj().task() {
                if let Some(operation) = task.current_operation() {
                    operation.status().has_no_detailed_progress()
                } else {
                    false
                }
            } else {
                false
            };

            // Show a pulse animation when there's a operation with no progress reporting
            if no_detailed_progress && !self.is_pulsing.get() {
                self.is_pulsing.set(true);
                glib::timeout_add_local(
                    Duration::from_secs(1),
                    clone!(@weak self as this => @default-return Continue(false), move || {
                        let is_pulsing = this.is_pulsing.get();

                        if is_pulsing {
                            this.progressbar.pulse();
                        } else {
                            this.update_fraction();
                        }

                        Continue(is_pulsing)
                    }),
                );
            } else {
                self.is_pulsing.set(false);
            }
        }

        fn update_fraction(&self) {
            let animation = self.animation.get().unwrap();

            if let Some(task) = self.obj().task() {
                animation.skip();
                let current_value = animation.value();

                animation.reset();
                animation.set_value_from(current_value);
                animation.set_value_to(task.progress() as f64);
                animation.play();
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTaskProgressBar(
        ObjectSubclass<imp::SkTaskProgressBar>)
        @extends gtk::Widget, adw::Bin;
}

impl SkTaskProgressBar {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for SkTaskProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
