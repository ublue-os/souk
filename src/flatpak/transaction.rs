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

use std::cell::Cell;

use glib::{ParamFlags, ParamSpec, ParamSpecFloat, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::worker::Progress;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTransaction {
        pub uuid: OnceCell<String>,
        pub progress: Cell<f32>,
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
                    ParamSpecFloat::new(
                        "progress",
                        "Progress",
                        "Progress",
                        0.0,
                        1.0,
                        0.0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "progress" => obj.progress().to_value(),
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
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkTransaction(ObjectSubclass<imp::SkTransaction>);
}

impl SkTransaction {
    pub fn new(uuid: &str) -> Self {
        glib::Object::new(&[("uuid", &uuid)]).unwrap()
    }

    pub fn uuid(&self) -> String {
        self.imp().uuid.get().unwrap().to_string()
    }

    pub fn progress(&self) -> f32 {
        self.imp().progress.get()
    }

    pub fn update(&self, progress: &Progress) {
        let imp = self.imp();

        imp.progress.set(progress.progress as f32 / 100.0);
        self.notify("progress");
    }
}
