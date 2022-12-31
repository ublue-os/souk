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
use crate::main::flatpak::package::SkPackage;
use crate::main::flatpak::sideload::SkSideloadType;
use crate::main::task::SkTask;
use crate::main::worker::SkWorker;
use crate::worker::DryRunResult;

// TODO: Refactor this into something similar to PackageInfo <-> SkPackage
// This should have a id, similar to PackageInfo, RemoteInfo, ...
// The counterpart to PackageInfo probably would be DryRunResult?
mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,

        /// TODO-REMOVE: dry_run_result which gets targeted during the sideload
        /// process
        pub dry_run_result: OnceCell<Option<SideloadPackage>>,

        /// Package which gets installed during the sideload process
        pub package: OnceCell<Option<SkPackage>>,
        /// Remotes which are getting added during the sideload process
        pub remotes: OnceCell<Vec<SkRemote>>,

        pub no_changes: OnceCell<bool>,
        pub installation: OnceCell<SkInstallation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadable {
        const NAME: &'static str = "SkSideloadable";
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

        // TODO-REMOVE: dry_run_result
        let sideload_package = SideloadPackage {
            dry_run_result: dry_run_result.clone(),
        };
        imp.dry_run_result.set(Some(sideload_package)).unwrap();

        // package
        let package = SkPackage::new(&dry_run_result.package);
        imp.package.set(Some(package)).unwrap();

        // remotes
        let mut remotes = Vec::new();
        for remote_info in &dry_run_result.added_remotes {
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
        imp.dry_run_result.set(None).unwrap();
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

    pub fn dry_run_result(&self) -> Option<SideloadPackage> {
        self.imp().dry_run_result.get().unwrap().to_owned()
    }

    pub fn package(&self) -> Option<SkPackage> {
        self.imp().package.get().unwrap().to_owned()
    }

    pub fn remotes(&self) -> Vec<SkRemote> {
        self.imp().remotes.get().unwrap().to_owned()
    }

    pub fn no_changes(&self) -> bool {
        *self.imp().no_changes.get().unwrap()
    }

    pub async fn sideload(&self, worker: &SkWorker) -> Result<Option<SkTask>, Error> {
        if let Some(dry_run_result) = self.dry_run_result() {
            let uninstall_before_install = dry_run_result.is_replacing_remote().is_some();

            let task = match self.type_() {
                SkSideloadType::Bundle => {
                    let task = worker
                        .install_flatpak_bundle_file(
                            &self.file(),
                            &self.installation(),
                            uninstall_before_install,
                            false,
                        )
                        .await?;
                    Some(task)
                }
                SkSideloadType::Ref => {
                    let task = worker
                        .install_flatpak_ref_file(
                            &self.file(),
                            &self.installation(),
                            uninstall_before_install,
                            false,
                        )
                        .await?;
                    Some(task)
                }
                _ => None,
            };

            return Ok(task);
        } else if self.type_() == SkSideloadType::Repo {
            let remotes = self.remotes();
            // There can be only *one* Flatpak repository in a *.flatpakrepo file
            let remote = remotes.first().unwrap();
            self.installation().add_remote(remote)?;
        }
        Ok(None)
    }
}
