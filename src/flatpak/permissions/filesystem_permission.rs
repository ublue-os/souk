// Souk - filesystem_permission.rs
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

use super::SkFilesystemPermissionType;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkFilesystemPermission {
        pub type_: OnceCell<SkFilesystemPermissionType>,
        pub path: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkFilesystemPermission {
        const NAME: &'static str = "SkFilesystemPermission";
        type ParentType = glib::Object;
        type Type = super::SkFilesystemPermission;
    }

    impl ObjectImpl for SkFilesystemPermission {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkFilesystemPermissionType::static_type(),
                        SkFilesystemPermissionType::ReadOnly as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("path", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "path" => obj.path().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkFilesystemPermission(ObjectSubclass<imp::SkFilesystemPermission>);
}

impl SkFilesystemPermission {
    pub fn new(value: &str) -> Self {
        let perm: Self = glib::Object::new(&[]).unwrap();
        let imp = perm.imp();

        let path: &str;
        let type_ = if value.ends_with(":rw") {
            path = value.trim_end_matches(":rw");
            SkFilesystemPermissionType::ReadWrite
        } else if value.ends_with(":create") {
            path = value.trim_end_matches(":create");
            SkFilesystemPermissionType::Create
        } else {
            path = value.trim_end_matches(":ro");
            SkFilesystemPermissionType::ReadOnly
        };

        imp.type_.set(type_).unwrap();
        imp.path.set(path.to_string()).unwrap();

        perm
    }

    pub fn type_(&self) -> SkFilesystemPermissionType {
        *self.imp().type_.get().unwrap()
    }

    pub fn path(&self) -> String {
        self.imp().path.get().unwrap().to_string()
    }
}
