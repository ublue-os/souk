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
use gtk::gio::File;
use gtk::glib;
use gtk::glib::Bytes;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libflatpak::Ref;
use once_cell::unsync::OnceCell;

use crate::error::Error;
use crate::flatpak::sideload::SkSideloadType;
use crate::flatpak::{SkTransaction, SkWorker};
use crate::worker::TransactionDryRun;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,
        pub transaction_dry_run: OnceCell<TransactionDryRun>,
        pub installation_uuid: OnceCell<String>,
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
    pub fn new(
        file: &File,
        type_: SkSideloadType,
        transaction_dry_run: TransactionDryRun,
        installation_uuid: &str,
    ) -> Self {
        let sideloadable: Self = glib::Object::new(&[]).unwrap();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(type_).unwrap();
        imp.transaction_dry_run.set(transaction_dry_run).unwrap();
        imp.installation_uuid
            .set(installation_uuid.to_string())
            .unwrap();

        sideloadable
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn type_(&self) -> SkSideloadType {
        *self.imp().type_.get().unwrap()
    }

    pub fn commit(&self) -> String {
        self.transaction_dry_run().commit.clone()
    }

    pub fn icon(&self) -> Option<gdk::Paintable> {
        let icon = self.transaction_dry_run().icon.clone();
        let bytes = Bytes::from_owned(icon);
        if let Ok(paintable) = gdk::Texture::from_bytes(&bytes) {
            Some(paintable.upcast())
        } else {
            None
        }
    }

    pub fn appstream_component(&self) -> Option<Component> {
        serde_json::from_str(&self.transaction_dry_run().appstream_component).ok()
    }

    pub fn installation_uuid(&self) -> String {
        self.imp().installation_uuid.get().unwrap().clone()
    }

    pub fn has_update_source(&self) -> bool {
        self.transaction_dry_run().has_update_source
    }

    pub fn is_replacing_remote(&self) -> String {
        self.transaction_dry_run().is_replacing_remote.clone()
    }

    pub fn contains_package(&self) -> bool {
        true
    }

    pub fn contains_repository(&self) -> bool {
        false // TODO: Bundles may include a repo
    }

    pub fn ref_(&self) -> Ref {
        let ref_ = self.transaction_dry_run().ref_.clone();
        Ref::parse(&ref_).unwrap()
    }

    pub fn is_already_done(&self) -> bool {
        self.transaction_dry_run().is_already_installed
    }

    pub fn is_update(&self) -> bool {
        self.transaction_dry_run().is_update
    }

    pub fn download_size(&self) -> u64 {
        self.transaction_dry_run().download_size
    }

    pub fn installed_size(&self) -> u64 {
        self.transaction_dry_run().installed_size
    }

    pub async fn sideload(&self, worker: &SkWorker) -> Result<Option<SkTransaction>, Error> {
        let no_update = !self.is_replacing_remote().is_empty();

        let transaction = match self.type_() {
            SkSideloadType::Bundle => {
                let transaction = worker
                    .install_flatpak_bundle(
                        &self.ref_(),
                        &self.file(),
                        &self.installation_uuid(),
                        no_update,
                    )
                    .await?;
                Some(transaction)
            }
            _ => None,
        };

        Ok(transaction)
    }

    fn transaction_dry_run(&self) -> &TransactionDryRun {
        self.imp().transaction_dry_run.get().unwrap()
    }
}
