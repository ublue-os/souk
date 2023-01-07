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
use std::str::FromStr;

use async_std::channel::{unbounded, Receiver, Sender};
use glib::subclass::Signal;
use glib::{
    ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecObject, ParamSpecString,
    ParamSpecUInt, ParamSpecUInt64, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::dry_run::SkDryRun;
use crate::main::flatpak::package::SkPackage;
use crate::main::task::{SkTaskActivity, SkTaskModel, SkTaskType};
use crate::shared::task::{Task, TaskProgress, TaskResponse, TaskResponseType, TaskResultType};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTask {
        pub data: OnceCell<Option<Task>>,

        // Static values
        pub index: OnceCell<u32>,
        pub type_: OnceCell<SkTaskType>,

        // Dynamic values
        pub package: RefCell<Option<SkPackage>>,
        pub activity: RefCell<SkTaskActivity>,
        pub progress: Cell<f32>,
        pub download_rate: Cell<u64>,

        pub dependencies: SkTaskModel,
        pub dependency_of: OnceCell<Option<super::SkTask>>,

        // Gets called when task finishes (done/error/cancelled)
        pub finished_sender: OnceCell<Sender<()>>,
        pub finished_receiver: OnceCell<Receiver<()>>,

        // Possible result values
        pub result_dry_run: OnceCell<SkDryRun>,
        pub result_error: OnceCell<String>,
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
                    ParamSpecUInt::new("index", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkTaskType::static_type(),
                        SkTaskType::default() as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "package",
                        "",
                        "",
                        SkPackage::static_type(),
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
                    ParamSpecObject::new(
                        "dependencies",
                        "",
                        "",
                        SkTaskModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "dependency-of",
                        "",
                        "",
                        super::SkTask::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "index" => obj.index().to_value(),
                "type" => obj.type_().to_value(),
                "package" => obj.package().to_value(),
                "activity" => obj.activity().to_value(),
                "progress" => obj.progress().to_value(),
                "download-rate" => obj.download_rate().to_value(),
                "dependencies" => obj.dependencies().to_value(),
                "dependency-of" => obj.dependency_of().to_value(),
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

        fn constructed(&self, _obj: &Self::Type) {
            let (finished_sender, finished_receiver) = unbounded();
            self.finished_sender.set(finished_sender).unwrap();
            self.finished_receiver.set(finished_receiver).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct SkTask(ObjectSubclass<imp::SkTask>);
}

impl SkTask {
    pub fn new(data: Task) -> Self {
        let task: Self = glib::Object::new(&[]).unwrap();
        let imp = task.imp();

        imp.data.set(Some(data.clone())).unwrap();
        imp.index.set(0).unwrap();
        imp.type_.set(SkTaskType::from_task_data(&data)).unwrap();

        imp.dependency_of.set(None).unwrap();

        task
    }

    /// Creates a new task which is a dependency of another task
    pub fn new_dependency(progress: &TaskProgress, dependency_of: &SkTask) -> Self {
        let task: Self = glib::Object::new(&[]).unwrap();
        let imp = task.imp();

        imp.data.set(None).unwrap();

        imp.index.set(progress.index).unwrap();
        imp.type_.set(progress.type_.clone().into()).unwrap();
        if let Some(package_info) = progress.package.clone().into() {
            let package = SkPackage::new(&package_info);
            *imp.package.borrow_mut() = Some(package);
        } else {
            *imp.package.borrow_mut() = None;
        }

        imp.dependency_of.set(Some(dependency_of.clone())).unwrap();

        task
    }

    pub fn uuid(&self) -> String {
        let task_data = self.data();

        if let Some(dependent_task) = self.dependency_of() {
            format!("{}-{}", dependent_task.uuid(), self.index())
        } else {
            task_data.uuid
        }
    }

    pub fn index(&self) -> u32 {
        *self.imp().index.get().unwrap()
    }

    pub fn type_(&self) -> SkTaskType {
        *self.imp().type_.get().unwrap()
    }

    pub fn package(&self) -> Option<SkPackage> {
        self.imp().package.borrow().clone()
    }

    pub fn activity(&self) -> SkTaskActivity {
        *self.imp().activity.borrow()
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn download_rate(&self) -> u64 {
        self.imp().download_rate.get()
    }

    pub fn dependencies(&self) -> SkTaskModel {
        self.imp().dependencies.clone()
    }

    pub fn dependency_of(&self) -> Option<SkTask> {
        self.imp().dependency_of.get().unwrap().clone()
    }

    pub fn handle_response(&self, response: &TaskResponse) {
        let imp = self.imp();

        match response.type_ {
            TaskResponseType::Initial => {
                let initial_response = response.initial_response.as_ref().unwrap();
                for task_progress in initial_response {
                    let is_last_task = task_progress.index as usize == (initial_response.len() - 1);

                    if self.type_().targets_single_package() && is_last_task {
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
                        let task = SkTask::new_dependency(task_progress, self);
                        self.dependencies().add_task(&task);
                        self.notify("dependencies");
                    }
                }
            }
            TaskResponseType::Update => {
                let update = response.update_response.as_ref().unwrap();
                let uuid = format!("{}-{}", self.uuid(), update.index);

                let is_last_task = update.index == self.dependencies().n_items();

                if let Some(task) = self.dependencies().task(&uuid) {
                    task.update(update);
                } else if !(self.type_().targets_single_package() && is_last_task) {
                    error!(
                        "Unable to retrieve dependency task for progress update: {}",
                        uuid
                    );
                }

                // Always update this task as well, so it mirrors the information of all
                // subtasks
                self.update(update);
            }
            TaskResponseType::Result => {
                let result = response.result_response.as_ref().unwrap();

                match result.type_ {
                    TaskResultType::Done => {
                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    TaskResultType::DoneDryRun => {
                        let dry_run = result.dry_run.as_ref().unwrap().clone();
                        let result_dry_run = SkDryRun::new(dry_run);
                        imp.result_dry_run.set(result_dry_run).unwrap();

                        imp.progress.set(1.0);
                        self.notify("progress");
                        self.emit_by_name::<()>("done", &[]);
                        imp.finished_sender.get().unwrap().try_send(()).unwrap();
                    }
                    TaskResultType::Error => {
                        let result_error = result.error.as_ref().unwrap().clone();
                        imp.result_error.set(result_error.clone()).unwrap();

                        self.emit_by_name::<()>("error", &[&result_error]);
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

    pub(super) fn update(&self, task_progress: &TaskProgress) {
        let imp = self.imp();

        let activity = SkTaskActivity::from_str(&task_progress.activity).unwrap();
        if self.activity() != activity {
            *imp.activity.borrow_mut() = activity;
            self.notify("activity");
        }

        // +1 since this task also counts to the total progress, and is not a dependency
        let total_tasks = self.dependencies().n_items() + 1;
        let mut task_index = task_progress.index;

        if self.dependencies().n_items() == 0 {
            task_index = 0;
        }
        let progress = ((task_index * 100) + task_progress.progress as u32) as f32
            / (total_tasks as f32 * 100.0);
        if progress != task_progress.progress as f32 {
            imp.progress.set(progress);
            self.notify("progress");
        }

        if self.download_rate() != task_progress.download_rate {
            imp.download_rate.set(task_progress.download_rate);
            self.notify("download-rate");
        }
    }

    pub async fn await_result(&self) -> Result<(), Error> {
        let imp = self.imp();
        imp.finished_receiver.get().unwrap().recv().await.unwrap();

        if let Some(err) = self.result_error() {
            Err(Error::Task(err))
        } else {
            Ok(())
        }
    }

    pub fn result_dry_run(&self) -> Option<SkDryRun> {
        self.imp().result_dry_run.get().cloned()
    }

    pub fn result_error(&self) -> Option<String> {
        self.imp().result_error.get().cloned()
    }

    /// Returns the original shared [Task] struct. If this task is a dependency
    /// to another task, the [Task] struct from the main [SkTask] gets
    /// returned.
    pub fn data(&self) -> Task {
        if let Some(dependent_task) = self.dependency_of() {
            dependent_task.data()
        } else {
            self.imp().data.get().unwrap().clone().unwrap()
        }
    }
}
