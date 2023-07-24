// Souk - task.rs
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

use async_std::channel::{unbounded, Receiver, Sender};
use glib::subclass::Signal;
use glib::{ParamSpec, Properties};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::dry_run::SkDryRun;
use crate::main::task::{SkOperation, SkOperationModel, SkTaskKind, SkTaskStatus};
use crate::shared::task::response::{OperationActivity, OperationStatus, TaskResult};
use crate::shared::task::Task;
use crate::shared::WorkerError;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkTask)]
    pub struct SkTask {
        #[property(get, set, construct_only)]
        #[property(name = "uuid", get = Self::uuid, type = String)]
        data: OnceCell<Task>,
        #[property(get, set, construct_only, builder(SkTaskKind::None))]
        kind: OnceCell<SkTaskKind>,

        #[property(get, builder(SkTaskStatus::Pending))]
        pub status: RefCell<SkTaskStatus>,
        #[property(get)]
        pub progress: Cell<f32>,

        #[property(get)]
        operations: SkOperationModel,
        #[property(get)]
        pub current_operation: RefCell<Option<SkOperation>>,

        // Gets called when task finishes (done/error/cancelled)
        pub finished_sender: OnceCell<Sender<()>>,
        pub finished_receiver: OnceCell<Receiver<()>>,

        // Possible result values
        pub result_dry_run: OnceCell<SkDryRun>,
        pub result_error: OnceCell<WorkerError>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTask {
        const NAME: &'static str = "SkTask";
        type Type = super::SkTask;
    }

    impl ObjectImpl for SkTask {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    // Activated, regardless of the result of the task
                    Signal::builder("completed").build(),
                    // Activated, when this task completed successfully
                    Signal::builder("done").build(),
                    Signal::builder("cancelled").build(),
                    Signal::builder("error")
                        .param_types([WorkerError::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();
            *self.status.borrow_mut() = SkTaskStatus::Pending;

            let (finished_sender, finished_receiver) = unbounded();
            self.finished_sender.set(finished_sender).unwrap();
            self.finished_receiver.set(finished_receiver).unwrap();
        }
    }

    impl SkTask {
        fn uuid(&self) -> String {
            self.obj().data().uuid
        }
    }
}

glib::wrapper! {
    pub struct SkTask(ObjectSubclass<imp::SkTask>);
}

impl SkTask {
    pub fn new(data: &Task) -> Self {
        let kind = SkTaskKind::from_task_data(data);

        glib::Object::builder()
            .property("data", data)
            .property("kind", kind)
            .build()
    }

    pub fn handle_activity(&self, activity: &Vec<OperationActivity>) {
        let imp = self.imp();
        self.operations().handle_activity(activity);

        // Change status from `Pending` to `Processing` on first activity
        *imp.status.borrow_mut() = SkTaskStatus::Processing;
        self.notify_status();

        // Find currently active ongoing operation of the task
        for oa in activity {
            if oa.status != OperationStatus::Done
                && oa.status != OperationStatus::Pending
                && oa.status != OperationStatus::None
            {
                let operation = self.operations().operation(&oa.identifier());

                if self.current_operation() != operation {
                    *imp.current_operation.borrow_mut() = operation;
                    self.notify_current_operation();
                }
                break;
            }
        }

        // Task percentage / progress
        let ops = self.operations().n_items();
        let mut i = 0.0;
        for op in self.operations().snapshot() {
            i += op.downcast::<SkOperation>().unwrap().progress();
        }

        let progress = i / (ops as f32);
        imp.progress.set(progress);
        self.notify_progress();
    }

    pub fn handle_result(&self, result: &TaskResult) {
        let imp = self.imp();

        let status = match result {
            TaskResult::Done => {
                imp.progress.set(1.0);
                self.notify_progress();
                self.emit_by_name::<()>("done", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();

                SkTaskStatus::Done
            }
            TaskResult::DoneDryRun(dry_run) => {
                let result_dry_run = SkDryRun::new(*dry_run.clone());
                imp.result_dry_run.set(result_dry_run).unwrap();

                imp.progress.set(1.0);
                self.notify_progress();
                self.emit_by_name::<()>("done", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();

                SkTaskStatus::Done
            }
            TaskResult::Error(worker_error) => {
                imp.result_error.set(*worker_error.clone()).unwrap();

                self.emit_by_name::<()>("error", &[&*(*worker_error)]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();

                SkTaskStatus::Error
            }
            TaskResult::Cancelled => {
                self.emit_by_name::<()>("cancelled", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();

                SkTaskStatus::Cancelled
            }
            _ => {
                warn!("Unknown response type");
                SkTaskStatus::None
            }
        };

        *imp.status.borrow_mut() = status;
        self.notify_status();

        self.emit_by_name::<()>("completed", &[]);
    }

    pub async fn await_result(&self) -> Result<(), Error> {
        let imp = self.imp();
        imp.finished_receiver.get().unwrap().recv().await.unwrap();

        if let Some(err) = self.result_error() {
            Err(Error::Worker(err))
        } else {
            Ok(())
        }
    }

    pub fn result_dry_run(&self) -> Option<SkDryRun> {
        self.imp().result_dry_run.get().cloned()
    }

    pub fn result_error(&self) -> Option<WorkerError> {
        self.imp().result_error.get().cloned()
    }
}
