// Shortwave - context_model.rs
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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::flatpak::context::SkContextDetail;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContextDetailModel {
        pub map: RefCell<IndexMap<String, SkContextDetail>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetailModel {
        const NAME: &'static str = "SkContextDetailModel";
        type Type = super::SkContextDetailModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkContextDetailModel {}

    impl ListModelImpl for SkContextDetailModel {
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
    pub struct SkContextDetailModel(ObjectSubclass<imp::SkContextDetailModel>) @implements gio::ListModel;
}

impl SkContextDetailModel {
    pub fn new(details: &[SkContextDetail]) -> Self {
        let model: Self = glib::Object::new(&[]).unwrap();

        let imp = model.imp();
        for (pos, detail) in details.iter().enumerate() {
            imp.map.borrow_mut().insert(pos.to_string(), detail.clone());
        }

        model
    }
}
