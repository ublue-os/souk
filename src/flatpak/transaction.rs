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

use glib::subclass::Signal;
use glib::{
    ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecFloat, ParamSpecInt, ParamSpecObject,
    ParamSpecString, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::Ref;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::flatpak::SkTransactionType;
use crate::worker::{Error, Progress};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTransaction {
        pub uuid: OnceCell<String>,

        pub ref_: OnceCell<Ref>,
        pub type_: OnceCell<SkTransactionType>,

        /// Name of Flatpak remote or bundle filename
        pub origin: OnceCell<String>,
        pub installation: OnceCell<String>,

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
                    ParamSpecString::new(
                        "uuid",
                        "UUID",
                        "UUID",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecObject::new(
                        "ref",
                        "Flatpak Ref",
                        "Flatpak Ref",
                        Ref::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecEnum::new(
                        "type",
                        "Type",
                        "Type",
                        SkTransactionType::static_type(),
                        SkTransactionType::None as i32,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "origin",
                        "Origin",
                        "Origin",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "installation",
                        "Installation",
                        "Installation",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecFloat::new(
                        "progress",
                        "Progress",
                        "Progress",
                        0.0,
                        1.0,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "current-operation-ref",
                        "Current Operation Ref",
                        "Current Operation Ref",
                        Ref::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "current-operation-type",
                        "Current Operation Type",
                        "Current Operation Type",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFloat::new(
                        "current-operation-progress",
                        "Current Operation Progress",
                        "Current Operation Progress",
                        0.0,
                        1.0,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecInt::new(
                        "current-operation",
                        "Current Operation",
                        "Current Operation",
                        0,
                        i32::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecInt::new(
                        "operations-count",
                        "Operations Count",
                        "Operations Count",
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
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "ref" => obj.ref_().to_value(),
                "type" => obj.type_().to_value(),
                "origin" => obj.origin().to_value(),
                "installation" => obj.installation().to_value(),
                "progress" => obj.progress().to_value(),
                "current-operation-ref" => obj.current_operation_ref().to_value(),
                "current-operation-type" => obj.current_operation_type().to_value(),
                "current-operation-progress" => obj.current_operation_progress().to_value(),
                "current-operation" => obj.current_operation().to_value(),
                "operations-count" => obj.operations_count().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "uuid" => self.uuid.set(value.get().unwrap()).unwrap(),
                "ref" => self.ref_.set(value.get().unwrap()).unwrap(),
                "type" => self.type_.set(value.get().unwrap()).unwrap(),
                "origin" => self.origin.set(value.get().unwrap()).unwrap(),
                "installation" => self.installation.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("done", &[], glib::Type::UNIT.into()).build(),
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
    }
}

glib::wrapper! {
    pub struct SkTransaction(ObjectSubclass<imp::SkTransaction>);
}

impl SkTransaction {
    pub fn new(
        uuid: &str,
        ref_: &Ref,
        type_: &SkTransactionType,
        origin: &str,
        installation: &str,
    ) -> Self {
        glib::Object::new(&[
            ("uuid", &uuid),
            ("ref", &ref_),
            ("type", &type_),
            ("origin", &origin),
            ("installation", &installation),
        ])
        .unwrap()
    }

    pub fn uuid(&self) -> String {
        self.imp().uuid.get().unwrap().to_string()
    }

    pub fn ref_(&self) -> Ref {
        self.imp().ref_.get().unwrap().clone()
    }

    pub fn type_(&self) -> SkTransactionType {
        *self.imp().type_.get().unwrap()
    }

    pub fn origin(&self) -> String {
        self.imp().origin.get().unwrap().to_string()
    }

    pub fn installation(&self) -> String {
        self.imp().installation.get().unwrap().to_string()
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

    pub(super) fn handle_progress(&self, progress: &Progress) {
        let imp = self.imp();

        if progress.is_done {
            imp.progress.set(1.0);
            self.notify("progress");

            self.emit_by_name::<()>("done", &[]);
            return;
        }

        let global_progress = (progress.current_operation as f32 - 1.0
            + (progress.progress as f32 / 100.0))
            / progress.operations_count as f32;
        imp.progress.set(global_progress as f32);
        self.notify("progress");

        let ref_ = Ref::parse(&progress.ref_).unwrap();
        *imp.current_operation_ref.borrow_mut() = Some(ref_);
        self.notify("current-operation-ref");

        *imp.current_operation_type.borrow_mut() = progress.type_.clone();
        self.notify("current-operation-type");

        let p = progress.progress as f32 / 100.0;
        imp.current_operation_progress.set(p);
        self.notify("current-operation-progress");

        imp.current_operation.set(progress.current_operation);
        self.notify("current-operation");

        imp.operations_count.set(progress.operations_count);
        self.notify("operations-count");
    }

    pub(super) fn handle_error(&self, error: &Error) {
        self.emit_by_name::<()>("error", &[&error.message.to_value()]);
    }
}
