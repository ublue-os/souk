// Souk - transaction.rs
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
use flatpak::Ref;
use glib::subclass::Signal;
use glib::{
    ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecInt, ParamSpecObject,
    ParamSpecString, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::transaction::SkTransactionType;
use crate::shared::task::{Response, ResponseType, Task};
use crate::worker::DryRunResult;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTransaction {
        pub data: OnceCell<Task>,

        /// Name of Flatpak remote or bundle filename
        pub origin: OnceCell<String>,

        /// Cumulative progress of the whole transaction
        pub progress: Cell<f32>,

        /// Ref of current Flatpak operation (dependencies, locales, ...)
        pub current_operation_ref: RefCell<Option<Ref>>,
        /// Name of the current Flatpak operation (eg. "install",
        /// "install-bundle", "update", ...)
        pub current_operation_type: RefCell<String>,
        pub current_operation_progress: Cell<f32>,

        /// The current operation index
        pub current_operation: Cell<i32>,
        /// The total count of operations in the transaction
        pub operations_count: Cell<i32>,

        // Gets called when task finished (done/error/cancelled)
        pub finished_sender: OnceCell<Sender<()>>,
        pub finished_receiver: OnceCell<Receiver<()>>,
        pub dry_run_result: RefCell<Option<DryRunResult>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTransaction {
        const NAME: &'static str = "SkTransaction";
        type ParentType = glib::Object;
        type Type = super::SkTransaction;
    }

    impl ObjectImpl for SkTransaction {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("uuid", "", "", None, ParamFlags::READABLE),
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkTransactionType::static_type(),
                        SkTransactionType::None as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("origin", "", "", None, ParamFlags::READABLE),
                    ParamSpecFloat::new("progress", "", "", 0.0, 1.0, 0.0, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "current-operation-ref",
                        "",
                        "",
                        Ref::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "current-operation-type",
                        "",
                        "",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFloat::new(
                        "current-operation-progress",
                        "",
                        "",
                        0.0,
                        1.0,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecInt::new(
                        "current-operation",
                        "",
                        "",
                        0,
                        i32::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecInt::new(
                        "operations-count",
                        "",
                        "",
                        0,
                        i32::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            // TODO: Should we re-add the installation info?
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "type" => obj.type_().to_value(),
                "origin" => obj.origin().to_value(),
                "progress" => obj.progress().to_value(),
                "current-operation-ref" => obj.current_operation_ref().to_value(),
                "current-operation-type" => obj.current_operation_type().to_value(),
                "current-operation-progress" => obj.current_operation_progress().to_value(),
                "current-operation" => obj.current_operation().to_value(),
                "operations-count" => obj.operations_count().to_value(),
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
    pub struct SkTransaction(ObjectSubclass<imp::SkTransaction>);
}

impl SkTransaction {
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

    pub fn type_(&self) -> SkTransactionType {
        SkTransactionType::None
    }

    pub fn origin(&self) -> String {
        self.imp().origin.get().unwrap().to_string()
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn current_operation_ref(&self) -> Option<Ref> {
        self.imp().current_operation_ref.borrow().as_ref().cloned()
    }

    pub fn current_operation_type(&self) -> String {
        self.imp().current_operation_type.borrow().to_string()
    }

    pub fn current_operation_progress(&self) -> f32 {
        self.imp().current_operation_progress.get()
    }

    pub fn current_operation(&self) -> i32 {
        self.imp().current_operation.get()
    }

    pub fn operations_count(&self) -> i32 {
        self.imp().operations_count.get()
    }

    pub fn handle_response(&self, response: &Response) {
        let imp = self.imp();

        match response.type_ {
            ResponseType::Done => {
                if let Some(flatpak_response) = response.flatpak_response() {
                    if let Some(dry_run_result) = flatpak_response.dry_run_result.into() {
                        *imp.dry_run_result.borrow_mut() = Some(dry_run_result);
                    }

                    imp.progress.set(flatpak_response.progress as f32 / 100.0);
                    self.notify("progress");
                }

                imp.progress.set(1.0);
                self.notify("progress");
                self.emit_by_name::<()>("done", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();
            }
            ResponseType::Update => {
                if let Some(flatpak_response) = response.flatpak_response() {
                    imp.progress.set(flatpak_response.progress as f32 / 100.0);
                    self.notify("progress");
                }
            }
            ResponseType::Cancelled => {
                self.emit_by_name::<()>("cancelled", &[]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();
            }
            ResponseType::Error => {
                let error = response.error_response().unwrap();
                self.emit_by_name::<()>("error", &[&error]);
                imp.finished_sender.get().unwrap().try_send(()).unwrap();
            }
        }

        //*imp.current_operation_type.borrow_mut() = progress.type_.clone();
        // self.notify("current-operation-type");

        // let p = response.progress as f32 / 100.0;
        // imp.current_operation_progress.set(p);
        // self.notify("current-operation-progress");

        // imp.current_operation.set(progress.current_operation);
        // self.notify("current-operation");

        // imp.operations_count.set(progress.operations_count);
        // self.notify("operations-count");
    }

    // TODO: Make this generic for all result types
    pub async fn await_dry_run_result(&self) -> Option<DryRunResult> {
        let imp = self.imp();
        imp.finished_receiver.get().unwrap().recv().await.unwrap();
        imp.dry_run_result.borrow().to_owned()
    }
}
