// Souk - context_detail_group_model.rs
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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::main::context::SkContextDetailGroup;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContextDetailGroupModel {
        pub map: RefCell<IndexMap<String, SkContextDetailGroup>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetailGroupModel {
        const NAME: &'static str = "SkContextDetailGroupModel";
        type Type = super::SkContextDetailGroupModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkContextDetailGroupModel {}

    impl ListModelImpl for SkContextDetailGroupModel {
        fn item_type(&self) -> glib::Type {
            SkContextDetailGroup::static_type()
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
    pub struct SkContextDetailGroupModel(ObjectSubclass<imp::SkContextDetailGroupModel>) @implements gio::ListModel;
}

impl SkContextDetailGroupModel {
    pub fn new(details: &[SkContextDetailGroup]) -> Self {
        let model: Self = glib::Object::new(&[]);

        let imp = model.imp();
        for (pos, detail) in details.iter().enumerate() {
            imp.map.borrow_mut().insert(pos.to_string(), detail.clone());
        }

        model
    }
}
