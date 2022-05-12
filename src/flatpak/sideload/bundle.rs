// Souk - bundle.rs
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

use gio::File;
use glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecUInt64, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{BundleRef, Ref};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkBundle {
        pub ref_: OnceCell<Ref>,
        pub file: OnceCell<File>,
        pub installed_size: OnceCell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkBundle {
        const NAME: &'static str = "SkBundle";
        type ParentType = glib::Object;
        type Type = super::SkBundle;
    }

    impl ObjectImpl for SkBundle {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "ref",
                        "Flatpak Ref",
                        "Flatpak Ref",
                        Ref::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecObject::new(
                        "file",
                        "File",
                        "File",
                        File::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecUInt64::new(
                        "installed-size",
                        "Installed Size",
                        "Installed Size",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "ref" => obj.ref_().to_value(),
                "file" => obj.file().to_value(),
                "installed-size" => obj.installed_size().to_value(),
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
                "ref" => self.ref_.set(value.get().unwrap()).unwrap(),
                "file" => self.file.set(value.get().unwrap()).unwrap(),
                "installed-size" => self.installed_size.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkBundle(ObjectSubclass<imp::SkBundle>);
}

impl SkBundle {
    pub fn new(bundle: &BundleRef) -> Self {
        let ref_: Ref = bundle.clone().upcast();
        let file = bundle.file().unwrap();
        let installed_size = bundle.installed_size();

        glib::Object::new(&[
            ("ref", &ref_),
            ("file", &file),
            ("installed-size", &installed_size),
        ])
        .unwrap()
    }

    pub fn ref_(&self) -> Ref {
        self.imp().ref_.get().unwrap().clone()
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn installed_size(&self) -> u64 {
        *self.imp().installed_size.get().unwrap()
    }
}
