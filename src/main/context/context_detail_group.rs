// Souk - context_detail_group.rs
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

use std::cell::RefCell;
use std::convert::TryInto;

use glib::{ParamSpec, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;
use once_cell::unsync::OnceCell;

use crate::main::context::SkContextDetail;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkContextDetailGroup)]
    pub struct SkContextDetailGroup {
        #[property(get, set, construct_only)]
        pub title: OnceCell<Option<String>>,
        #[property(get, set, construct_only)]
        pub description: OnceCell<Option<String>>,

        pub map: RefCell<IndexMap<String, SkContextDetail>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetailGroup {
        const NAME: &'static str = "SkContextDetailGroup";
        type Type = super::SkContextDetailGroup;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkContextDetailGroup {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
    }

    impl ListModelImpl for SkContextDetailGroup {
        fn item_type(&self) -> glib::Type {
            SkContextDetail::static_type()
        }

        fn n_items(&self) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
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
    pub fn new(title: Option<&str>, description: Option<&str>) -> Self {
        glib::Object::builder()
            .property("title", &title)
            .property("description", &description)
            .build()
    }

    pub fn add_details(&self, details: &[SkContextDetail]) {
        let imp = self.imp();
        for (pos, detail) in details.iter().enumerate() {
            imp.map.borrow_mut().insert(pos.to_string(), detail.clone());
        }
    }
}
