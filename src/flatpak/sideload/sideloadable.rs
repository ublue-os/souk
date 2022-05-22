// Souk - sideloadable.rs
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

use core::fmt::Debug;

use async_trait::async_trait;
use glib::{
    ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject, ParamSpecUInt64,
    ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::prelude::*;
use libflatpak::Ref;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::app::SkApplication;
use crate::error::Error;
use crate::flatpak::sideload::SkSideloadType;
use crate::flatpak::{SkTransaction, SkWorker};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub sideloadable: OnceCell<Box<dyn Sideloadable>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadable {
        const NAME: &'static str = "SkSideloadable";
        type ParentType = glib::Object;
        type Type = super::SkSideloadable;
    }

    impl ObjectImpl for SkSideloadable {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "type",
                        "Type",
                        "Type",
                        SkSideloadType::static_type(),
                        SkSideloadType::None as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "contains-package",
                        "Contains Package",
                        "Contains Package",
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "contains-repository",
                        "Contains Repository",
                        "Contains Repository",
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "ref",
                        "Ref",
                        "Ref",
                        Ref::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "is-already-done",
                        "Is Already Done",
                        "Is Already Done",
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "is-update",
                        "Is Update",
                        "Is Update",
                        false,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt64::new(
                        "download-size",
                        "Download Size",
                        "Download Size",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt64::new(
                        "installed-size",
                        "Installed Size",
                        "Installed Size",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "contains-package" => obj.contains_package().to_value(),
                "contains-repository" => obj.contains_repository().to_value(),
                "ref" => obj.ref_().to_value(),
                "is-already-done" => obj.is_already_done().to_value(),
                "is-update" => obj.is_update().to_value(),
                "download-size" => obj.download_size().to_value(),
                "installed-size" => obj.installed_size().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkSideloadable(ObjectSubclass<imp::SkSideloadable>);
}

impl SkSideloadable {
    pub fn new(sideloadable: Box<dyn Sideloadable>) -> Self {
        let obj: Self = glib::Object::new(&[]).unwrap();

        let imp = obj.imp();
        imp.sideloadable.set(sideloadable).unwrap();

        obj
    }

    pub fn type_(&self) -> SkSideloadType {
        self.imp().sideloadable.get().unwrap().type_()
    }

    pub fn commit(&self) -> String {
        self.imp().sideloadable.get().unwrap().commit()
    }

    pub fn installation_uuid(&self) -> String {
        self.imp().sideloadable.get().unwrap().installation_uuid()
    }

    pub fn contains_package(&self) -> bool {
        self.imp().sideloadable.get().unwrap().contains_package()
    }

    pub fn contains_repository(&self) -> bool {
        self.imp().sideloadable.get().unwrap().contains_repository()
    }

    pub fn ref_(&self) -> Ref {
        self.imp().sideloadable.get().unwrap().ref_().upcast()
    }

    pub fn is_already_done(&self) -> bool {
        self.imp().sideloadable.get().unwrap().is_already_done()
    }

    pub fn is_update(&self) -> bool {
        self.imp().sideloadable.get().unwrap().is_update()
    }

    pub fn download_size(&self) -> u64 {
        self.imp().sideloadable.get().unwrap().download_size()
    }

    pub fn installed_size(&self) -> u64 {
        self.imp().sideloadable.get().unwrap().installed_size()
    }

    pub async fn sideload(&self) -> Result<SkTransaction, Error> {
        let worker = SkApplication::default().worker();
        self.imp()
            .sideloadable
            .get()
            .unwrap()
            .sideload(&worker)
            .await
    }
}

#[async_trait(?Send)]
pub trait Sideloadable {
    fn type_(&self) -> SkSideloadType;

    fn commit(&self) -> String;

    fn installation_uuid(&self) -> String;

    fn contains_package(&self) -> bool;

    fn contains_repository(&self) -> bool;

    fn ref_(&self) -> Ref;

    fn is_already_done(&self) -> bool;

    fn is_update(&self) -> bool;

    fn download_size(&self) -> u64;

    fn installed_size(&self) -> u64;

    async fn sideload(&self, worker: &SkWorker) -> Result<SkTransaction, Error>;
}

impl Debug for dyn Sideloadable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Sideloadable: {}", self.ref_().format_ref().unwrap())
    }
}
