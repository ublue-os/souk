// Shortwave - remote_model.rs
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

use std::cell::RefCell;
use std::convert::TryInto;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::flatpak::installation::SkRemote;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkRemoteModel {
        pub map: RefCell<IndexMap<String, SkRemote>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemoteModel {
        const NAME: &'static str = "SkRemoteModel";
        type Type = super::SkRemoteModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkRemoteModel {}

    impl ListModelImpl for SkRemoteModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkRemote::static_type()
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
    pub struct SkRemoteModel(ObjectSubclass<imp::SkRemoteModel>) @implements gio::ListModel;
}

impl SkRemoteModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_remote(&self, remote: &SkRemote) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&remote.id()) {
                warn!("Remote {:?} already exists in model", remote.id());
                return;
            }

            map.insert(remote.id(), remote.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_remote(&self, remote: &SkRemote) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            match map.get_index_of(&remote.id()) {
                Some(pos) => {
                    map.remove(&remote.id());
                    Some(pos)
                }
                None => {
                    warn!("Remote {:?} not found in model", remote.id());
                    None
                }
            }
        };

        if let Some(pos) = pos {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }
    }

    pub fn remote(&self, id: &str) -> Option<SkRemote> {
        self.imp().map.borrow().get(id).cloned()
    }
}

impl Default for SkRemoteModel {
    fn default() -> Self {
        Self::new()
    }
}
