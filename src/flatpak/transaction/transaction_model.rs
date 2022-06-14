// Shortwave - transaction_model.rs
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

use crate::flatpak::transaction::SkTransaction;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkTransactionModel {
        pub map: RefCell<IndexMap<String, SkTransaction>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTransactionModel {
        const NAME: &'static str = "SkTransactionModel";
        type Type = super::SkTransactionModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkTransactionModel {}

    impl ListModelImpl for SkTransactionModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SkTransaction::static_type()
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
    pub struct SkTransactionModel(ObjectSubclass<imp::SkTransactionModel>) @implements gio::ListModel;
}

impl SkTransactionModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_transaction(&self, transaction: &SkTransaction) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&transaction.uuid()) {
                warn!(
                    "Transaction {:?} already exists in model",
                    transaction.uuid()
                );
                return;
            }

            map.insert(transaction.uuid(), transaction.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_transaction(&self, transaction: &SkTransaction) {
        let mut map = self.imp().map.borrow_mut();

        match map.get_index_of(&transaction.uuid()) {
            Some(pos) => {
                map.remove(&transaction.uuid());
                self.items_changed(pos.try_into().unwrap(), 1, 0);
            }
            None => warn!("Transaction {:?} not found in model", transaction.uuid()),
        }
    }

    pub fn transaction(&self, uuid: &str) -> Option<SkTransaction> {
        self.imp().map.borrow().get(uuid).cloned()
    }
}

impl Default for SkTransactionModel {
    fn default() -> Self {
        Self::new()
    }
}
