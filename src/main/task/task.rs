// Souk - task.rs
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

use async_std::channel::{unbounded, Receiver, Sender};
use glib::subclass::Signal;
use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecObject, ParamSpecString,
    ParamSpecUInt64, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::task::{SkTaskActivity, SkTaskStep, SkTaskStepModel, SkTaskType};
use crate::shared::task::{Task, TaskResponse, TaskResponseType, TaskResultType};
use crate::worker::DryRunResult;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTask {
        pub data: OnceCell<Task>,

        /// Type of this task
        pub type_: OnceCell<SkTaskType>,
        /// Cumulative progress of the complete task (with all steps)
        pub progress: Cell<f32>,
        /// All steps of this task
        pub steps: SkTaskStepModel,

        download_rate_watch: OnceCell<gtk::ExpressionWatch>,
        activity_watch: OnceCell<gtk::ExpressionWatch>,

        // Gets called when task finished (done/error/cancelled)
        pub finished_sender: OnceCell<Sender<()>>,
        pub finished_receiver: OnceCell<Receiver<()>>,
        pub dry_run_result: RefCell<Option<DryRunResult>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTask {
        const NAME: &'static str = "SkTask";
        type Type = super::SkTask;
    }

    impl ObjectImpl for SkTask {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("uuid", "", "", None, ParamFlags::READABLE),
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkTaskType::static_type(),
                        SkTaskType::default() as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFloat::new("progress", "", "", 0.0, 1.0, 0.0, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "steps",
                        "",
                        "",
                        SkTaskStepModel::static_type(),
                        ParamFlags::READABLE,
                    ),
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
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "type" => obj.type_().to_value(),
                "progress" => obj.progress().to_value(),
                "steps" => obj.steps().to_value(),
                "download-rate" => obj.download_rate().to_value(),
                "activity" => obj.activity().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("done", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder("cancelled", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder(
                        "error",
                        &[glib::Type::STRING.into()],
                        glib::Type::UNIT.into(),
                    )
                    .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            let watch = self
                .steps
                .property_expression("current")
                .chain_property::<SkTaskStep>("download-rate")
                .watch(
                    glib::Object::NONE,
                    clone!(@weak obj => move|| obj.notify("download-rate")),
                );
            self.download_rate_watch.set(watch).unwrap();

            let watch = self
                .steps
                .property_expression("current")
                .chain_property::<SkTaskStep>("activity")
                .watch(
                    glib::Object::NONE,
                    clone!(@weak obj => move|| obj.notify("activity")),
                );
            self.activity_watch.set(watch).unwrap();

            let (finished_sender, finished_receiver) = unbounded();
            self.finished_sender.set(finished_sender).unwrap();
            self.finished_receiver.set(finished_receiver).unwrap();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.download_rate_watch.get().unwrap().unwatch();
            self.activity_watch.get().unwrap().unwatch();
        }
    }
}

glib::wrapper! {
    pub struct SkTask(ObjectSubclass<imp::SkTask>);
}

impl SkTask {
    pub fn new(data: Task) -> Self {
        let task: Self = glib::Object::new(&[]).unwrap();
        task.imp().data.set(data).unwrap();

        task
    }

    /// Returns the original shared [Task] struct
    pub fn data(&self) -> Task {
        self.imp().data.get().unwrap().clone()
    }

    pub fn uuid(&self) -> String {
        self.imp().data.get().unwrap().uuid.clone()
    }

    pub fn type_(&self) -> SkTaskType {
        SkTaskType::None
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn steps(&self) -> SkTaskStepModel {
        self.imp().steps.clone()
    }

    /// Returns the download rate of the current active [SkTaskStep]
    pub fn download_rate(&self) -> u64 {
        if let Some(step) = self.steps().current() {
            step.download_rate()
        } else {
            0
        }
    }

    /// Returns the activity of the current active [SkTaskStep]
    pub fn activity(&self) -> SkTaskActivity {
        if let Some(step) = self.steps().current() {
            step.activity()
        } else {
            SkTaskActivity::None
        }
    }

    pub fn handle_response(&self, response: &TaskResponse) {
        let imp = self.imp();

        match response.type_ {
            TaskResponseType::Initial => {
                let steps = response.initial_response.as_ref().unwrap();
                self.steps().set_steps(steps);
            }
            TaskResponseType::Update => {
                let updated_step = response.update_response.as_ref().unwrap();
                self.steps().update_step(Some(updated_step));

                let progress = ((updated_step.index * 100) + updated_step.progress as u32) as f32
                    / (self.steps().n_items() as f32 * 100.0);
                dbg!(&progress);
                if self.progress() != progress {
                    self.imp().progress.set(progress);
                    self.notify("progress");
                }
            }
            TaskResponseType::Result => {
                let result = response.result_response.as_ref().unwrap();

                // Unset `current` property of SkTaskStepModel
                self.steps().update_step(None);

                match result.type_ {
                    TaskResultType::Done => {
                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    TaskResultType::DoneDryRun => {
                        let dry_run_result = result.dry_run.as_ref().unwrap().clone();
                        *imp.dry_run_result.borrow_mut() = Some(dry_run_result);

                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    TaskResultType::Error => {
                        let error = result.error.as_ref().unwrap().clone();
                        self.emit_by_name::<()>("error", &[&error]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    TaskResultType::Cancelled => {
                        self.emit_by_name::<()>("cancelled", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    _ => warn!("Unknown response type"),
                }
            }
        }
    }

    // TODO: Make this generic for all result types
    // Do we need to expose the raw DryRunResult object? Why we can't return the
    // SkSideloadable?
    pub async fn await_dry_run_result(&self) -> Option<DryRunResult> {
        let imp = self.imp();
        imp.finished_receiver.get().unwrap().recv().await.unwrap();
        imp.dry_run_result.borrow().to_owned()
    }
}
