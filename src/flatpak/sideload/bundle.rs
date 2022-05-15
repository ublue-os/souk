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

use async_trait::async_trait;
use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecUInt64, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::{BundleRef, Ref};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::error::Error;
use crate::flatpak::sideload::{Sideloadable, SkSideloadType};
use crate::flatpak::{SkTransaction, SkWorker};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkBundle {
        pub ref_: OnceCell<BundleRef>,
        pub already_done: OnceCell<bool>,
        pub download_size: OnceCell<u64>,
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
                    ParamSpecBoolean::new(
                        "already-done",
                        "Already Done",
                        "Already Done",
                        false,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecUInt64::new(
                        "download-size",
                        "Download Size",
                        "Download Size",
                        0,
                        u64::MAX,
                        0,
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
                "already-done" => obj.already_done().to_value(),
                "download-size" => obj.download_size().to_value(),
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
                "already-done" => self.already_done.set(value.get().unwrap()).unwrap(),
                "download-size" => self.download_size.set(value.get().unwrap()).unwrap(),
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
    pub fn new(ref_: &BundleRef, download_size: u64, installed_size: u64) -> Self {
        glib::Object::new(&[
            ("ref", &ref_),
            ("download-size", &download_size),
            ("installed-size", &installed_size),
        ])
        .unwrap()
    }
}

#[async_trait(?Send)]
impl Sideloadable for SkBundle {
    fn type_(&self) -> SkSideloadType {
        SkSideloadType::Bundle
    }

    fn contains_package(&self) -> bool {
        true
    }

    fn contains_repository(&self) -> bool {
        false // TODO: Bundles may include a repo
    }

    fn ref_(&self) -> Ref {
        self.imp().ref_.get().unwrap().clone().upcast()
    }

    fn already_done(&self) -> bool {
        *self.imp().already_done.get().unwrap()
    }

    fn download_size(&self) -> u64 {
        *self.imp().download_size.get().unwrap()
    }

    fn installed_size(&self) -> u64 {
        *self.imp().installed_size.get().unwrap()
    }

    async fn sideload(
        &self,
        worker: &SkWorker,
        installation: &str,
    ) -> Result<SkTransaction, Error> {
        let imp = self.imp();
        let bundle_ref = imp.ref_.get().unwrap();
        let transaction = worker
            .install_flatpak_bundle(bundle_ref, installation)
            .await?;
        Ok(transaction)
    }
}
