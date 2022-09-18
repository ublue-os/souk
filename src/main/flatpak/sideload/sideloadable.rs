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

use gtk::gio::File;
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use super::SideloadPackage;
use crate::main::error::Error;
use crate::main::flatpak::installation::{SkInstallation, SkRemote};
use crate::main::flatpak::sideload::SkSideloadType;
use crate::main::flatpak::transaction::SkTransaction;
use crate::main::flatpak::SkWorker;
use crate::worker::DryRunResult;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,

        /// Package which gets targeted during the sideload process
        pub package: OnceCell<Option<SideloadPackage>>,
        /// Remotes which are getting added during the sideload process
        pub remotes: OnceCell<Vec<SkRemote>>,

        pub no_changes: OnceCell<bool>,
        pub installation: OnceCell<SkInstallation>,
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
        installation: &SkInstallation,
    ) -> Self {
        let sideloadable: Self = glib::Object::new(&[]).unwrap();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(type_).unwrap();
        imp.no_changes
            .set(dry_run_result.is_already_installed)
            .unwrap();
        imp.installation.set(installation.clone()).unwrap();

        // package
        let package = SideloadPackage {
            dry_run_result: dry_run_result.clone(),
        };
        imp.package.set(Some(package)).unwrap();

        // remotes
        let mut remotes = Vec::new();
        for remote_info in &dry_run_result.remotes_info {
            let remote = SkRemote::new(remote_info);
            remotes.push(remote);
        }
        imp.remotes.set(remotes).unwrap();

        sideloadable
    }

    /// For '*.flatpakrepo' file
    pub fn new_repo(
        file: &File,
        remote: &SkRemote,
        already_added: bool,
        installation: &SkInstallation,
    ) -> Self {
        let sideloadable: Self = glib::Object::new(&[]).unwrap();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(SkSideloadType::Repo).unwrap();
        imp.package.set(None).unwrap();
        imp.remotes.set(vec![remote.clone()]).unwrap();
        imp.no_changes.set(already_added).unwrap();
        imp.installation.set(installation.clone()).unwrap();

        sideloadable
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn type_(&self) -> SkSideloadType {
        *self.imp().type_.get().unwrap()
    }

    pub fn installation(&self) -> SkInstallation {
        self.imp().installation.get().unwrap().clone()
    }

    pub fn package(&self) -> Option<SideloadPackage> {
        self.imp().package.get().unwrap().to_owned()
    }

    pub fn remotes(&self) -> Vec<SkRemote> {
        self.imp().remotes.get().unwrap().to_owned()
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
                            &self.installation(),
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
                            &self.installation(),
                            no_update,
                        )
                        .await?;
                    Some(transaction)
                }
                _ => None,
            };

            return Ok(transaction);
        } else if self.type_() == SkSideloadType::Repo {
            let remotes = self.remotes();
            // There can be only *one* Flatpak repository in a *.flatpakrepo file
            let remote = remotes.first().unwrap();
            self.installation().add_remote(remote)?;
        }
        Ok(None)
    }
}
