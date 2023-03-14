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
use crate::main::flatpak::package::SkPackage;
use crate::main::task::{SkTaskKind, SkTaskModel, SkTaskStatus};
use crate::shared::task::response::{TaskResponse, TaskResponseKind, TaskResultKind, TaskUpdate};
use crate::shared::task::Task;
use crate::shared::WorkerError;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkTask)]
    pub struct SkTask {
        // Static values
        #[property(get)]
        index: OnceCell<u32>,
        #[property(get, builder(SkTaskKind::None))]
        kind: OnceCell<SkTaskKind>,

        // Dynamic values
        #[property(get)]
        pub package: RefCell<Option<SkPackage>>,
        #[property(get, builder(SkTaskStatus::Pending))]
        pub status: RefCell<SkTaskStatus>,
        #[property(get)]
        pub progress: Cell<f32>,
        #[property(get)]
        download_rate: Cell<u64>,

        #[property(get)]
        dependencies: SkTaskModel,
        #[property(get, set, construct_only)]
        dependency_of: OnceCell<Option<super::SkTask>>,

        #[property(get = Self::data, set, construct_only, type = Task)]
        #[property(name = "uuid", get = Self::uuid, type = String)]
        data: OnceCell<Option<Task>>,

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

            // Only set the task kind for ¬dependency tasks, since those get the kind set
            // from the [TaskResponseKind::Initial] response.
            if let Some(data) = self.data.get().unwrap() {
                self.kind.set(SkTaskKind::from_task_data(data)).unwrap();
            }

            *self.status.borrow_mut() = SkTaskStatus::Pending;

            let (finished_sender, finished_receiver) = unbounded();
            self.finished_sender.set(finished_sender).unwrap();
            self.finished_receiver.set(finished_receiver).unwrap();
        }
    }

    impl SkTask {
        /// Returns the original shared [Task] struct. If this task is a
        /// dependency to another task, the [Task] struct from the main
        /// [SkTask] gets returned.
        fn data(&self) -> Task {
            if let Some(dependent_task) = self.obj().dependency_of() {
                dependent_task.data()
            } else {
                self.data.get().unwrap().clone().unwrap()
            }
        }

        fn uuid(&self) -> String {
            let task_data = self.data();

            if let Some(dependent_task) = self.obj().dependency_of() {
                format!("{}:{}", dependent_task.uuid(), self.obj().index())
            } else {
                task_data.uuid
            }
        }

        /// Sets the initial data of a [TaskUpdate] which comes via a
        /// [TaskResponseKind::Initial] response
        pub fn set_initial(&self, initial: &TaskUpdate) {
            self.index.set(initial.index).unwrap();
            self.kind
                .set(initial.operation_kind.clone().into())
                .unwrap();

            if let Some(package_info) = initial.package.clone() {
                let package = SkPackage::new(&package_info);
                *self.package.borrow_mut() = Some(package);
            } else {
                *self.package.borrow_mut() = None;
            }
            self.update(initial);
        }

        pub fn update(&self, task_update: &TaskUpdate) {
            let status = SkTaskStatus::from(task_update.status.clone());
            if self.obj().status() != status {
                *self.status.borrow_mut() = status;
                self.obj().notify("status");
            }

            // +1 since this task also counts to the total progress, and is not a dependency
            let total_tasks = self.obj().dependencies().n_items() + 1;
            let mut task_index = task_update.index;

            if self.obj().dependencies().n_items() == 0 {
                task_index = 0;
            }
            let progress = ((task_index * 100) + task_update.progress as u32) as f32
                / (total_tasks as f32 * 100.0);
            if progress != task_update.progress as f32 {
                self.progress.set(progress);
                self.obj().notify("progress");
            }

            if self.obj().download_rate() != task_update.download_rate {
                self.download_rate.set(task_update.download_rate);
                self.obj().notify("download-rate");
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTask(ObjectSubclass<imp::SkTask>);
}

impl SkTask {
    pub fn new(data: Option<&Task>, dependency_of: Option<&SkTask>) -> Self {
        glib::Object::builder()
            .property("data", data)
            .property("dependency-of", dependency_of)
            .build()
    }

    pub fn handle_response(&self, response: &TaskResponse) {
        let imp = self.imp();

        match response.kind {
            TaskResponseKind::Initial => {
                let initial_response = response.initial_response.as_ref().unwrap();
                for task_progress in initial_response {
                    let is_last_task = task_progress.index as usize == (initial_response.len() - 1);

                    if self.kind().targets_single_package() && is_last_task {
                        // It only affects one particular package, and the update response has the
                        // last index -> it affects this task
                        if let Some(package) = task_progress.package.as_ref() {
                            let package = SkPackage::new(package);
                            *imp.package.borrow_mut() = Some(package);
                            self.notify("package");
                        }
                    } else {
                        // Check if the update is *not* for the last task (which would be `self`,
                        // and not a dependency), and if the task targets a single package.
                        let task = SkTask::new(None, Some(self));
                        task.imp().set_initial(task_progress);

                        self.dependencies().add_task(&task);
                        self.notify("dependencies");
                    }
                }
            }
            TaskResponseKind::Update => {
                let update = response.update_response.as_ref().unwrap();
                let uuid = format!("{}:{}", self.uuid(), update.index);

                let is_last_task = update.index == self.dependencies().n_items();

                if let Some(task) = self.dependencies().task(&uuid) {
                    task.imp().update(update);
                } else if !(self.kind().targets_single_package() && is_last_task) {
                    error!(
                        "Unable to retrieve dependency task for progress update: {}",
                        uuid
                    );
                }

                // Always update this task as well, so it mirrors the information of all
                // subtasks
                self.imp().update(update);
            }
            TaskResponseKind::Result => {
                let result = response.result_response.as_ref().unwrap();

                let status = match result.kind {
                    TaskResultKind::Done => {
                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                        SkTaskStatus::Done
                    }
                    TaskResultKind::DoneDryRun => {
                        let dry_run = result.dry_run.as_ref().unwrap().clone();
                        let result_dry_run = SkDryRun::new(dry_run);
                        imp.result_dry_run.set(result_dry_run).unwrap();

                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                        SkTaskStatus::Done
                    }
                    TaskResultKind::Error => {
                        let result_error = result.error.as_ref().unwrap().clone();
                        imp.result_error.set(result_error.clone()).unwrap();

                        self.emit_by_name::<()>("error", &[&result_error]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                        SkTaskStatus::Error
                    }
                    TaskResultKind::Cancelled => {
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
                self.notify("status");

                self.emit_by_name::<()>("completed", &[]);
            }
        }
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
