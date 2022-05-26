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

use appstream::Component;
use async_trait::async_trait;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::prelude::*;
use libflatpak::Ref;
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

    impl ObjectImpl for SkSideloadable {}
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

    pub fn icon(&self) -> Option<gdk::Paintable> {
        self.imp().sideloadable.get().unwrap().icon()
    }

    pub fn appstream_component(&self) -> Option<Component> {
        self.imp().sideloadable.get().unwrap().appstream_component()
    }

    pub fn installation_uuid(&self) -> String {
        self.imp().sideloadable.get().unwrap().installation_uuid()
    }

    pub fn has_update_source(&self) -> bool {
        self.imp().sideloadable.get().unwrap().has_update_source()
    }

    pub fn is_replacing_remote(&self) -> String {
        self.imp().sideloadable.get().unwrap().is_replacing_remote()
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

    fn icon(&self) -> Option<gdk::Paintable>;

    fn appstream_component(&self) -> Option<Component>;

    fn installation_uuid(&self) -> String;

    fn has_update_source(&self) -> bool;

    fn is_replacing_remote(&self) -> String;

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
