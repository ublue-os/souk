// Souk - service_permission.rs
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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, ToValue};
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkServicePermission {
        pub name: OnceCell<String>,
        pub is_system: OnceCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkServicePermission {
        const NAME: &'static str = "SkServicePermission";
        type ParentType = glib::Object;
        type Type = super::SkServicePermission;
    }

    impl ObjectImpl for SkServicePermission {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-system", "", "", false, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "is-system" => obj.is_system().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkServicePermission(ObjectSubclass<imp::SkServicePermission>);
}

impl SkServicePermission {
    pub fn new(name: &str, is_system: bool) -> Self {
        let perm: Self = glib::Object::new(&[]).unwrap();

        let imp = perm.imp();
        imp.name.set(name.to_string()).unwrap();
        imp.is_system.set(is_system).unwrap();

        perm
    }

    pub fn name(&self) -> String {
        self.imp().name.get().unwrap().to_string()
    }

    pub fn is_system(&self) -> bool {
        *self.imp().is_system.get().unwrap()
    }
}
