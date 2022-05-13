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
use glib::{ParamFlags, ParamSpec, ParamSpecObject, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::prelude::*;
use libflatpak::{BundleRef, Ref};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use zbus::Result;

use crate::flatpak::sideload::{Sideloadable, SkSideloadType};
use crate::flatpak::{SkTransaction, SkWorker};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkBundle {
        pub ref_: OnceCell<BundleRef>,
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
                vec![ParamSpecObject::new(
                    "ref",
                    "Flatpak Ref",
                    "Flatpak Ref",
                    Ref::static_type(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "ref" => obj.ref_().to_value(),
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
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkBundle(ObjectSubclass<imp::SkBundle>);
}

impl SkBundle {
    pub fn new(ref_: &BundleRef) -> Self {
        glib::Object::new(&[("ref", &ref_)]).unwrap()
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

    fn installed_size(&self) -> u64 {
        self.imp().ref_.get().unwrap().installed_size()
    }

    async fn sideload(&self, worker: &SkWorker, installation: &str) -> Result<SkTransaction> {
        let imp = self.imp();
        let bundle_ref = imp.ref_.get().unwrap();
        worker
            .install_flatpak_bundle(bundle_ref, installation)
            .await
    }
}
