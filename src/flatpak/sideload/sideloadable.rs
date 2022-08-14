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

use flatpak::Remote;
use gtk::gio::File;
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use super::SideloadPackage;
use crate::error::Error;
use crate::flatpak::sideload::SkSideloadType;
use crate::flatpak::transaction::SkTransaction;
use crate::flatpak::SkWorker;
use crate::worker::DryRunResult;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,

        pub package: OnceCell<Option<SideloadPackage>>,
        pub remote: OnceCell<Option<Remote>>,

        pub no_changes: OnceCell<bool>,
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
    /// Can be a *.flatpakref or *.flatpak file
    pub fn new_package(
        file: &File,
        type_: SkSideloadType,
        dry_run_result: DryRunResult,
        installation_uuid: &str,
    ) -> Self {
        let sideloadable: Self = glib::Object::new(&[]).unwrap();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(type_).unwrap();
        imp.no_changes
            .set(dry_run_result.is_already_installed)
            .unwrap();
        imp.installation_uuid
            .set(installation_uuid.to_string())
            .unwrap();

        // remote
        let remote = dry_run_result
            .new_remote
            .as_ref()
            .map(|r| r.flatpak_remote());
        imp.remote.set(remote).unwrap();

        // package
        let package = SideloadPackage { dry_run_result };
        imp.package.set(Some(package)).unwrap();

        sideloadable
    }

    // *.flatpak repo
    pub fn new_repo(
        file: &File,
        remote: &Remote,
        already_added: bool,
        installation_uuid: &str,
    ) -> Self {
        let sideloadable: Self = glib::Object::new(&[]).unwrap();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(SkSideloadType::Repo).unwrap();
        imp.package.set(None).unwrap();
        imp.remote.set(Some(remote.clone())).unwrap();
        imp.no_changes.set(already_added).unwrap();
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

    pub fn installation_uuid(&self) -> String {
        self.imp().installation_uuid.get().unwrap().clone()
    }

    pub fn package(&self) -> Option<SideloadPackage> {
        self.imp().package.get().unwrap().to_owned()
    }

    pub fn remote(&self) -> Option<Remote> {
        self.imp().remote.get().unwrap().to_owned()
    }

    pub fn no_changes(&self) -> bool {
        *self.imp().no_changes.get().unwrap()
    }

    pub async fn sideload(&self, worker: &SkWorker) -> Result<Option<SkTransaction>, Error> {
        if let Some(package) = self.package() {
            let no_update = package.is_replacing_remote().is_some();

            let transaction = match self.type_() {
                SkSideloadType::Bundle => {
                    let transaction = worker
                        .install_flatpak_bundle(
                            &package.ref_(),
                            &self.file(),
                            &self.installation_uuid(),
                            no_update,
                        )
                        .await?;
                    Some(transaction)
                }
                SkSideloadType::Ref => {
                    let transaction = worker
                        .install_flatpak_ref(
                            &package.ref_(),
                            &self.file(),
                            &self.installation_uuid(),
                            no_update,
                        )
                        .await?;
                    Some(transaction)
                }
                _ => None,
            };

            return Ok(transaction);
        } else if self.type_() == SkSideloadType::Repo {
            worker
                .add_remote(&self.file(), &self.installation_uuid())
                .await?;
        }
        Ok(None)
    }
}
