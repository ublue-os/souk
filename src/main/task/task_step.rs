// Souk - task_step.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{
    ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecObject, ParamSpecUInt, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

use crate::main::flatpak::package::SkPackage;
use crate::main::flatpak::sideload::SkSideloadable;
use crate::main::task::SkTaskActivity;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTaskStep {
        /// Progress of this step
        pub progress: Cell<f32>,
        /// Download rate of this step, in bytes per second (if something gets
        /// downloaded)
        pub download_rate: Cell<u32>,
        /// The current activity of this step
        pub activity: RefCell<SkTaskActivity>,
        /// The related package. This is only set if this is not a sideloading
        /// task.
        pub package: RefCell<Option<SkPackage>>,
        /// The related package. This is only set if this is a sideloading task.
        pub sideloadable: RefCell<Option<SkSideloadable>>,
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
                    ParamSpecFloat::new("progress", "", "", 0.0, 1.0, 0.0, ParamFlags::READABLE),
                    ParamSpecUInt::new(
                        "download-rate",
                        "",
                        "",
                        0,
                        u32::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "activity",
                        "",
                        "",
                        SkTaskActivity::static_type(),
                        SkTaskActivity::Waiting as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "package",
                        "",
                        "",
                        SkPackage::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "sideloadable",
                        "",
                        "",
                        SkSideloadable::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "progress" => obj.progress().to_value(),
                "download-rate" => obj.download_rate().to_value(),
                "activity" => obj.activity().to_value(),
                "package" => obj.package().to_value(),
                "sideloadable" => obj.sideloadable().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTaskStep(ObjectSubclass<imp::SkTaskStep>);
}

impl SkTaskStep {
    pub fn new() -> Self {
        let step: Self = glib::Object::new(&[]).unwrap();
        // task.imp().data.set(data).unwrap();

        step
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn download_rate(&self) -> u32 {
        self.imp().download_rate.get()
    }

    pub fn activity(&self) -> SkTaskActivity {
        *self.imp().activity.borrow()
    }

    pub fn package(&self) -> Option<SkPackage> {
        self.imp().package.borrow().as_ref().cloned()
    }

    pub fn sideloadable(&self) -> Option<SkSideloadable> {
        self.imp().sideloadable.borrow().as_ref().cloned()
    }

    // pub fn handle_response(&self, response: &Response) {
    // let imp = self.imp();
    //
    // match response.type_ {
    // ResponseType::Done => {
    // if let Some(flatpak_response) = response.flatpak_response() {
    // if let Some(dry_run_result) = flatpak_response.dry_run_result.into() {
    // imp.dry_run_result.borrow_mut() = Some(dry_run_result);
    // }
    //
    // imp.progress.set(flatpak_response.progress as f32 / 100.0);
    // self.notify("progress");
    // }
    //
    // imp.progress.set(1.0);
    // self.notify("progress");
    // self.emit_by_name::<()>("done", &[]);
    // imp.finished_sender.get().unwrap().try_send(()).unwrap();
    // }
    // ResponseType::Update => {
    // if let Some(flatpak_response) = response.flatpak_response() {
    // imp.progress.set(flatpak_response.progress as f32 / 100.0);
    // self.notify("progress");
    // }
    // }
    // ResponseType::Cancelled => {
    // self.emit_by_name::<()>("cancelled", &[]);
    // imp.finished_sender.get().unwrap().try_send(()).unwrap();
    // }
    // ResponseType::Error => {
    // let error = response.error_response().unwrap();
    // self.emit_by_name::<()>("error", &[&error]);
    // imp.finished_sender.get().unwrap().try_send(()).unwrap();
    // }
    // }
    //
    // imp.current_operation_type.borrow_mut() = progress.type_.clone();
    // self.notify("current-operation-type");
    //
    // let p = response.progress as f32 / 100.0;
    // imp.current_operation_progress.set(p);
    // self.notify("current-operation-progress");
    //
    // imp.current_operation.set(progress.current_operation);
    // self.notify("current-operation");
    //
    // imp.operations_count.set(progress.operations_count);
    // self.notify("operations-count");
    // }
}
