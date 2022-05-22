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
use crate::worker::DryRunResults;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkBundle {
        pub ref_: OnceCell<BundleRef>,
        pub dry_run_results: OnceCell<DryRunResults>,
        pub installation_uuid: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkBundle {
        const NAME: &'static str = "SkBundle";
        type ParentType = glib::Object;
        type Type = super::SkBundle;
    }

    impl ObjectImpl for SkBundle {}
}

glib::wrapper! {
    pub struct SkBundle(ObjectSubclass<imp::SkBundle>);
}

impl SkBundle {
    pub fn new(ref_: &BundleRef, dry_run_results: DryRunResults, installation_uuid: &str) -> Self {
        let bundle: Self = glib::Object::new(&[]).unwrap();

        let imp = bundle.imp();
        imp.ref_.set(ref_.clone()).unwrap();
        imp.dry_run_results.set(dry_run_results).unwrap();
        imp.installation_uuid.set(installation_uuid.into()).unwrap();

        bundle
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

    fn is_already_done(&self) -> bool {
        self.imp().dry_run_results.get().unwrap().is_already_done
    }

    fn is_update(&self) -> bool {
        self.imp().dry_run_results.get().unwrap().is_update
    }

    fn download_size(&self) -> u64 {
        self.imp().dry_run_results.get().unwrap().download_size
    }

    fn installed_size(&self) -> u64 {
        self.imp().dry_run_results.get().unwrap().installed_size
    }

    async fn sideload(&self, worker: &SkWorker) -> Result<SkTransaction, Error> {
        let imp = self.imp();

        let bundle_ref = imp.ref_.get().unwrap();
        let installation_uuid = imp.installation_uuid.get().unwrap();

        let transaction = worker
            .install_flatpak_bundle(bundle_ref, installation_uuid)
            .await?;
        Ok(transaction)
    }
}
