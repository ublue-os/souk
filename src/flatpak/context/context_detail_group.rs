// Shortwave - context_detail_group.rs
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

use std::cell::RefCell;
use std::convert::TryInto;

use glib::{ParamFlags, ParamSpec, ParamSpecString, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::flatpak::context::SkContextDetail;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContextDetailGroup {
        pub map: RefCell<IndexMap<String, SkContextDetail>>,

        pub description: OnceCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetailGroup {
        const NAME: &'static str = "SkContextDetailGroup";
        type Type = super::SkContextDetailGroup;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkContextDetailGroup {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "description",
                    "",
                    "",
                    None,
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
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
                "description" => self.description.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for SkContextDetailGroup {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkContextDetail::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SkContextDetailGroup(ObjectSubclass<imp::SkContextDetailGroup>) @implements gio::ListModel;
}

impl SkContextDetailGroup {
    pub fn new(details: &[SkContextDetail], description: Option<&str>) -> Self {
        let model: Self = glib::Object::new(&[("description", &description)]).unwrap();

        let imp = model.imp();
        for (pos, detail) in details.iter().enumerate() {
            imp.map.borrow_mut().insert(pos.to_string(), detail.clone());
        }

        model
    }

    pub fn description(&self) -> Option<String> {
        self.imp().description.get().unwrap().to_owned()
    }
}
