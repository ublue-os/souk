// Souk - task_step.rs
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

use std::cell::{Cell, RefCell};
use std::str::FromStr;

use glib::{
    ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecObject, ParamSpecUInt,
    ParamSpecUInt64, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::package::SkPackage;
use crate::main::task::SkTaskActivity;
use crate::shared::task::TaskProgress;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskStep {
        /// Index of this step
        pub index: OnceCell<u32>,
        /// Progress of this step
        pub progress: Cell<f32>,
        /// Download rate of this step, in bytes per second (if something gets
        /// downloaded)
        pub download_rate: Cell<u64>,
        /// The *current* activity of this step (note: it's the *current*
        /// activity, not the targeted activity, so it doesn't have to match the
        /// Flatpak operation name, eg. because it's "pending")
        pub activity: RefCell<SkTaskActivity>,
        /// The related package. This is only set if this is not a sideloading
        /// task.
        pub package: OnceCell<Option<SkPackage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTaskStep {
        const NAME: &'static str = "SkTaskStep";
        type Type = super::SkTaskStep;
    }

    impl ObjectImpl for SkTaskStep {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecUInt::new("index", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                    ParamSpecFloat::new("progress", "", "", 0.0, 1.0, 0.0, ParamFlags::READABLE),
                    ParamSpecUInt64::new(
                        "download-rate",
                        "",
                        "",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "activity",
                        "",
                        "",
                        SkTaskActivity::static_type(),
                        SkTaskActivity::default() as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "package",
                        "",
                        "",
                        SkPackage::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "index" => obj.index().to_value(),
                "progress" => obj.progress().to_value(),
                "download-rate" => obj.download_rate().to_value(),
                "activity" => obj.activity().to_value(),
                "package" => obj.package().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTaskStep(ObjectSubclass<imp::SkTaskStep>);
}

impl SkTaskStep {
    pub fn new(task_step: &TaskProgress) -> Self {
        let step: Self = glib::Object::new(&[]).unwrap();
        let imp = step.imp();

        imp.index.set(task_step.index).unwrap();
        *imp.activity.borrow_mut() = SkTaskActivity::from_str(&task_step.activity).unwrap();

        if let Some(package_info) = task_step.package_info.clone().into() {
            let package = SkPackage::new(&package_info);
            imp.package.set(Some(package)).unwrap();
        } else {
            imp.package.set(None).unwrap();
        }

        step
    }

    pub(super) fn update(&self, task_step: &TaskProgress) {
        let imp = self.imp();

        if self.progress() != task_step.progress as f32 {
            imp.progress.set(task_step.progress as f32 / 100.0);
            self.notify("progress");
        }

        if self.download_rate() != task_step.download_rate {
            imp.download_rate.set(task_step.download_rate);
            self.notify("download-rate");
        }

        let activity = SkTaskActivity::from_str(&task_step.activity).unwrap();
        if self.activity() != activity {
            *imp.activity.borrow_mut() = activity;
            self.notify("activity");
        }
    }

    pub fn index(&self) -> u32 {
        *self.imp().index.get().unwrap()
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn download_rate(&self) -> u64 {
        self.imp().download_rate.get()
    }

    pub fn activity(&self) -> SkTaskActivity {
        *self.imp().activity.borrow()
    }

    pub fn package(&self) -> Option<SkPackage> {
        self.imp().package.get().unwrap().clone()
    }
}
