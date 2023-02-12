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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject, ToValue};
use gtk::gio::File;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::dry_run::SkDryRun;
use crate::main::flatpak::installation::{SkInstallation, SkRemote, SkRemoteModel};
use crate::main::flatpak::sideload::SkSideloadType;
use crate::main::flatpak::SkFlatpakOperationType;
use crate::main::task::SkTask;
use crate::main::worker::SkWorker;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkSideloadable {
        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,

        /// Package which gets installed during sideload process (evaluated as
        /// [SkDryRun])
        pub dry_run: OnceCell<Option<SkDryRun>>,

        /// Remotes which are getting added during the sideload process
        pub remotes: OnceCell<SkRemoteModel>,

        pub no_changes: OnceCell<bool>,
        pub installation: OnceCell<SkInstallation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadable {
        const NAME: &'static str = "SkSideloadable";
        type Type = super::SkSideloadable;
    }

    impl ObjectImpl for SkSideloadable {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new("file", "", "", File::static_type(), ParamFlags::READABLE),
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkSideloadType::static_type(),
                        SkSideloadType::None as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "dry-run",
                        "",
                        "",
                        SkDryRun::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "remotes",
                        "",
                        "",
                        SkRemoteModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new("no-changes", "", "", false, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "installation",
                        "",
                        "",
                        SkInstallation::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.obj().file().to_value(),
                "type" => self.obj().type_().to_value(),
                "dry-run" => self.obj().dry_run().to_value(),
                "remotes" => self.obj().remotes().to_value(),
                "no-changes" => self.obj().no_changes().to_value(),
                "installation" => self.obj().installation().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkSideloadable(ObjectSubclass<imp::SkSideloadable>);
}

impl SkSideloadable {
    /// Can be a *.flatpakref or *.flatpak file
    pub fn new_package(
        file: &File,
        type_: SkSideloadType,
        dry_run: SkDryRun,
        installation: &SkInstallation,
    ) -> Self {
        let sideloadable: Self = glib::Object::new();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(type_).unwrap();
        imp.no_changes
            .set(dry_run.package().operation_type() == SkFlatpakOperationType::None)
            .unwrap();
        imp.installation.set(installation.clone()).unwrap();

        // remotes
        imp.remotes.set(dry_run.remotes()).unwrap();

        // package dry run
        imp.dry_run.set(Some(dry_run)).unwrap();

        sideloadable
    }

    /// For '*.flatpakrepo' file
    pub fn new_repo(
        file: &File,
        remote: &SkRemote,
        already_added: bool,
        installation: &SkInstallation,
    ) -> Self {
        let sideloadable: Self = glib::Object::new();

        let imp = sideloadable.imp();
        imp.file.set(file.clone()).unwrap();
        imp.type_.set(SkSideloadType::Repo).unwrap();
        imp.dry_run.set(None).unwrap();
        imp.no_changes.set(already_added).unwrap();
        imp.installation.set(installation.clone()).unwrap();

        let remotes = SkRemoteModel::new();
        remotes.set_remotes(vec![remote.info()]);
        imp.remotes.set(remotes).unwrap();

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

    pub fn dry_run(&self) -> Option<SkDryRun> {
        self.imp().dry_run.get().unwrap().to_owned()
    }

    pub fn remotes(&self) -> SkRemoteModel {
        self.imp().remotes.get().unwrap().to_owned()
    }

    pub fn no_changes(&self) -> bool {
        *self.imp().no_changes.get().unwrap()
    }

    pub async fn sideload(&self, worker: &SkWorker) -> Result<Option<SkTask>, Error> {
        if let Some(dry_run) = self.dry_run() {
            let uninstall_before_install = dry_run.is_replacing_remote().is_some();

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
            let remote: SkRemote = remotes.item(0).unwrap().downcast().unwrap();
            self.installation().add_remote(&remote)?;
        }
        Ok(None)
    }
}
