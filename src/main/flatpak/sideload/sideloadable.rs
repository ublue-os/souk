// Souk - sideloadable.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{ParamSpec, Properties};
use gtk::gio::File;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::dry_run::SkDryRun;
use crate::main::flatpak::installation::{SkInstallation, SkRemote, SkRemoteModel};
use crate::main::flatpak::sideload::SkSideloadKind;
use crate::main::flatpak::SkFlatpakOperationType;
use crate::main::task::SkTask;
use crate::main::worker::SkWorker;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkSideloadable)]
    pub struct SkSideloadable {
        #[property(get, set, construct_only)]
        pub file: OnceCell<File>,
        #[property(get, set, construct_only, builder(SkSideloadKind::None))]
        pub kind: OnceCell<SkSideloadKind>,
        /// Package which gets installed during sideload process (evaluated as
        /// [SkDryRun])
        #[property(get, set, construct_only)]
        pub dry_run: OnceCell<Option<SkDryRun>>,
        /// Remotes which are getting added during the sideload process
        #[property(get, set, construct_only)]
        pub remotes: OnceCell<SkRemoteModel>,
        #[property(get, set, construct_only)]
        pub no_changes: OnceCell<bool>,
        #[property(get, set, construct_only)]
        pub installation: OnceCell<SkInstallation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadable {
        const NAME: &'static str = "SkSideloadable";
        type Type = super::SkSideloadable;
    }

    impl ObjectImpl for SkSideloadable {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
    }
}

glib::wrapper! {
    pub struct SkSideloadable(ObjectSubclass<imp::SkSideloadable>);
}

impl SkSideloadable {
    pub fn new(
        file: &File,
        kind: &SkSideloadKind,
        dry_run: Option<&SkDryRun>,
        remotes: &SkRemoteModel,
        no_changes: bool,
        installation: &SkInstallation,
    ) -> Self {
        glib::Object::builder()
            .property("file", file)
            .property("kind", kind)
            .property("dry-run", dry_run)
            .property("remotes", remotes)
            .property("no-changes", no_changes)
            .property("installation", installation)
            .build()
    }

    /// Can be a *.flatpakref or *.flatpak file
    pub fn new_package(
        file: &File,
        kind: &SkSideloadKind,
        dry_run: &SkDryRun,
        installation: &SkInstallation,
    ) -> Self {
        Self::new(
            file,
            kind,
            Some(dry_run),
            &dry_run.remotes(),
            dry_run.package().operation_type() == SkFlatpakOperationType::None,
            installation,
        )
    }

    /// For '*.flatpakrepo' file
    pub fn new_repo(
        file: &File,
        remote: &SkRemote,
        already_added: bool,
        installation: &SkInstallation,
    ) -> Self {
        let remotes = SkRemoteModel::new();
        remotes.set_remotes(vec![remote.info()]);

        Self::new(
            file,
            &SkSideloadKind::Repo,
            None,
            &remotes,
            already_added,
            installation,
        )
    }

    pub async fn sideload(&self, worker: &SkWorker) -> Result<Option<SkTask>, Error> {
        if let Some(dry_run) = self.dry_run() {
            let uninstall_before_install = dry_run.is_replacing_remote().is_some();

            let task = match self.kind() {
                SkSideloadKind::Bundle => {
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
                SkSideloadKind::Ref => {
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
        } else if self.kind() == SkSideloadKind::Repo {
            let remotes = self.remotes();
            // There can be only *one* Flatpak repository in a *.flatpakrepo file
            let remote: SkRemote = remotes.item(0).unwrap().downcast().unwrap();
            self.installation().add_remote(&remote)?;
        }
        Ok(None)
    }
}
