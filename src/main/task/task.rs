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
use crate::shared::task::response::{TaskActivity, TaskResult};
use crate::shared::task::Task;
use crate::shared::WorkerError;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkTask)]
    pub struct SkTask {
        // Static values
        #[property(get = Self::data, set, construct_only, type = Task)]
        #[property(name = "uuid", get = Self::uuid, type = String)]
        data: OnceCell<Option<Task>>,

        #[property(get, set, construct_only)]
        pub index: OnceCell<u32>,
        #[property(get, set, construct_only, builder(SkTaskKind::None))]
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
            if let Some(dependent_task) = self.obj().dependency_of() {
                format!("{}:{}", dependent_task.uuid(), self.obj().index())
            } else {
                let task_data = self.data();
                task_data.uuid
            }
        }

        pub fn update(&self, activity: &TaskActivity) {
            // status
            let status = SkTaskStatus::from(activity.status.clone());
            if self.obj().status() != status {
                *self.status.borrow_mut() = status;
                self.obj().notify("status");
            }

            // progress
            let progress = activity.progress as f32 / 100.0;
            if self.obj().progress() != progress {
                self.progress.set(progress);
                self.obj().notify("progress");
            }

            // download rate
            if self.obj().download_rate() != activity.download_rate {
                self.download_rate.set(activity.download_rate);
                self.obj().notify("download-rate");
            }

            // package
            let package = activity.package.as_ref().map(SkPackage::new);
            if self.obj().package() != package {
                self.package.set(package);
                self.obj().notify("package");
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTask(ObjectSubclass<imp::SkTask>);
}

impl SkTask {
    pub fn new(
        data: Option<&Task>,
        activity: Option<&TaskActivity>,
        dependency_of: Option<&SkTask>,
    ) -> Self {
        let index = if let Some(activity) = activity {
            activity.index
        } else {
            0
        };

        let kind = if let Some(kind) = data {
            SkTaskKind::from_task_data(kind)
        } else if let Some(activity) = activity {
            SkTaskKind::from(activity.operation_kind.clone())
        } else {
            SkTaskKind::Unknown
        };

        let task: Self = glib::Object::builder()
            .property("data", data)
            .property("index", index)
            .property("kind", kind)
            .property("dependency-of", dependency_of)
            .build();

        if let Some(activity) = activity {
            task.handle_activity(activity);
        }

        task
    }

    pub fn handle_activity(&self, activity: &TaskActivity) {
        let imp = self.imp();
        imp.update(activity);

        for dependency in &activity.dependencies {
            let uuid = format!("{}:{}", self.uuid(), dependency.index);
            if let Some(task) = self.dependencies().task(&uuid) {
                task.handle_activity(dependency);
            } else {
                let task = Self::new(None, Some(dependency), Some(self));
                self.dependencies().add_task(&task);
            }
        }
    }

    pub fn handle_result(&self, result: &TaskResult) {
        let imp = self.imp();

        let status = match result {
            TaskResult::Done => {
                imp.progress.set(1.0);
                self.notify("progress");
                self.emit_by_name::<()>("done", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();
                SkTaskStatus::Done
            }
            TaskResult::DoneDryRun(dry_run) => {
                let result_dry_run = SkDryRun::new(dry_run.clone());
                imp.result_dry_run.set(result_dry_run).unwrap();

                imp.progress.set(1.0);
                self.notify("progress");
                self.emit_by_name::<()>("done", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();
                SkTaskStatus::Done
            }
            TaskResult::Error(worker_error) => {
                imp.result_error.set(worker_error.clone()).unwrap();

                self.emit_by_name::<()>("error", &[&worker_error]);
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
        self.notify("status");

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
