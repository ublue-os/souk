// Souk - context_detail.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::flatpak::context::{SkContextDetailLevel, SkContextDetailType};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContextDetail {
        pub type_: OnceCell<SkContextDetailType>,
        pub type_value: OnceCell<String>,
        pub level: OnceCell<SkContextDetailLevel>,
        pub title: OnceCell<String>,
        pub description: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetail {
        const NAME: &'static str = "SkContextDetail";
        type ParentType = glib::Object;
        type Type = super::SkContextDetail;
    }

    impl ObjectImpl for SkContextDetail {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkContextDetailType::static_type(),
                        SkContextDetailType::ICON as i32,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "type-value",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecEnum::new(
                        "level",
                        "",
                        "",
                        SkContextDetailLevel::static_type(),
                        SkContextDetailLevel::NEUTRAL as i32,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "title",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "description",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "type-value" => obj.type_value().to_value(),
                "level" => obj.level().to_value(),
                "title" => obj.title().to_value(),
                "description" => obj.description().to_value(),
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
                "type" => self.type_.set(value.get().unwrap()).unwrap(),
                "type-value" => self.type_value.set(value.get().unwrap()).unwrap(),
                "level" => self.level.set(value.get().unwrap()).unwrap(),
                "title" => self.title.set(value.get().unwrap()).unwrap(),
                "description" => self.description.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkContextDetail(ObjectSubclass<imp::SkContextDetail>);
}

impl SkContextDetail {
    pub fn new(
        type_: SkContextDetailType,
        type_value: &str,
        level: SkContextDetailLevel,
        title: &str,
        description: &str,
    ) -> Self {
        glib::Object::new(&[
            ("type", &type_),
            ("type-value", &type_value),
            ("level", &level),
            ("title", &title),
            ("description", &description),
        ])
        .unwrap()
    }

    pub fn new_neutral_size(size: u64, title: &str, description: &str) -> Self {
        glib::Object::new(&[
            ("type", &SkContextDetailType::SIZE),
            ("type-value", &size.to_string()),
            ("level", &SkContextDetailLevel::NEUTRAL),
            ("title", &title),
            ("description", &description),
        ])
        .unwrap()
    }

    pub fn new_neutral_text(text: &str, title: &str, description: &str) -> Self {
        glib::Object::new(&[
            ("type", &SkContextDetailType::TEXT),
            ("type-value", &text.to_string()),
            ("level", &SkContextDetailLevel::NEUTRAL),
            ("title", &title),
            ("description", &description),
        ])
        .unwrap()
    }

    pub fn type_(&self) -> SkContextDetailType {
        *self.imp().type_.get().unwrap()
    }

    pub fn type_value(&self) -> String {
        self.imp().type_value.get().unwrap().to_string()
    }

    pub fn level(&self) -> SkContextDetailLevel {
        *self.imp().level.get().unwrap()
    }

    pub fn title(&self) -> String {
        self.imp().title.get().unwrap().to_string()
    }

    pub fn description(&self) -> String {
        self.imp().description.get().unwrap().to_string()
    }
}
