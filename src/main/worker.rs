// Souk - worker.rs
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

use flatpak::Remote;
use futures_util::stream::StreamExt;
use gio::File;
use glib::{clone, KeyFile, ParamSpec, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::main::dbus_proxy::WorkerProxy;
use crate::main::error::Error;
use crate::main::flatpak::installation::{SkInstallation, SkInstallationModel, SkRemote};
use crate::main::flatpak::package::SkPackage;
use crate::main::flatpak::sideload::{SkSideloadKind, SkSideloadable};
use crate::main::flatpak::utils;
use crate::main::task::{SkTask, SkTaskModel};
use crate::shared::flatpak::info::RemoteInfo;
use crate::shared::task::FlatpakTask;

/// Number of tasks that are completed and still remain in log
const KEEP_COMPLETED_TASKS: u32 = 5;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkWorker)]
    pub struct SkWorker {
        #[property(get)]
        tasks: SkTaskModel,
        #[property(get)]
        installations: SkInstallationModel,

        pub proxy: WorkerProxy<'static>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorker {
        const NAME: &'static str = "SkWorker";
        type Type = super::SkWorker;
    }

    impl ObjectImpl for SkWorker {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let fut = clone!(@weak self as this => async move {
                this.receive_task_response().await;
            });
            gtk_macros::spawn!(fut);
        }
    }

    impl SkWorker {
        pub async fn run_task(&self, task: &SkTask) -> Result<(), Error> {
            // Remove finished tasks from model
            task.connect_local(
                "completed",
                false,
                clone!(@weak self as this => @default-return None, move |_|{
                    this.obj().tasks().remove_completed_tasks(KEEP_COMPLETED_TASKS);
                    None
                }),
            );

            self.obj().tasks().add_task(task);
            self.proxy.run_task(task.data()).await?;

            Ok(())
        }

        /// Handle incoming task responses from worker process
        pub async fn receive_task_response(&self) {
            let mut response = self.proxy.receive_task_response().await.unwrap();

            while let Some(response) = response.next().await {
                let response = response.args().unwrap().task_response;
                debug!("Task response: {:#?}", response);

                let task_uuid = response.uuid.clone();
                match self.obj().tasks().task(&task_uuid) {
                    Some(task) => task.handle_response(&response),
                    None => warn!("Received response for unknown active task!"),
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct SkWorker(ObjectSubclass<imp::SkWorker>);
}

impl SkWorker {
    /// Install new Flatpak by ref name
    pub async fn install_flatpak(
        &self,
        package: SkPackage,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTask, Error> {
        let task_data =
            FlatpakTask::new_install(&package.info(), uninstall_before_install, dry_run);

        let task = SkTask::new(Some(&task_data), None);
        self.imp().run_task(&task).await?;

        Ok(task)
    }

    /// Install new Flatpak by bundle file
    pub async fn install_flatpak_bundle_file(
        &self,
        file: &File,
        installation: &SkInstallation,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTask, Error> {
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let task_data = FlatpakTask::new_install_bundle_file(
            &installation.info(),
            &path_string,
            uninstall_before_install,
            dry_run,
        );

        let task = SkTask::new(Some(&task_data), None);
        self.imp().run_task(&task).await?;

        Ok(task)
    }

    /// Install new Flatpak by ref file
    pub async fn install_flatpak_ref_file(
        &self,
        file: &File,
        installation: &SkInstallation,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTask, Error> {
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let task_data = FlatpakTask::new_install_ref_file(
            &installation.info(),
            &path_string,
            uninstall_before_install,
            dry_run,
        );

        let task = SkTask::new(Some(&task_data), None);
        self.imp().run_task(&task).await?;

        Ok(task)
    }

    /// Cancel a worker task
    pub async fn cancel_task(&self, task: &SkTask) -> Result<(), Error> {
        self.imp().proxy.cancel_task(task.data()).await?;
        Ok(())
    }

    /// Opens a sideloadable Flatpak file and load it into a `SkSideloadable`
    /// which can be viewed / installed in a `SkSideloadWindow`
    pub async fn load_sideloadable(
        &self,
        file: &File,
        installation: &SkInstallation,
    ) -> Result<SkSideloadable, Error> {
        let kind = SkSideloadKind::determine_type(file);

        let task = match kind {
            SkSideloadKind::Bundle => {
                self.install_flatpak_bundle_file(file, installation, false, true)
                    .await?
            }
            SkSideloadKind::Ref => {
                self.install_flatpak_ref_file(file, installation, false, true)
                    .await?
            }
            SkSideloadKind::Repo => {
                let bytes = file.load_bytes(gio::Cancellable::NONE)?.0;

                let keyfile = KeyFile::new();
                keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;
                let key = keyfile.value("Flatpak Repo", "GPGKey")?;

                // Flatpak needs a name for the remote. Try using the `Title` value for it,
                // otherwise fall back to the filename.
                let remote_name = if let Ok(title) = keyfile.value("Flatpak Repo", "Title") {
                    utils::normalize_string(&title)
                } else {
                    // Should be safe to unwrap here, since we don't accept files without an
                    // extension at all
                    let basename = file.basename().unwrap();
                    utils::normalize_string(&basename.to_string_lossy())
                };

                let flatpak_remote = Remote::from_file(&remote_name, &bytes)?;
                let mut remote_info = RemoteInfo::from_flatpak(&flatpak_remote, None);
                remote_info.set_gpg_key(&key);
                let sk_remote = SkRemote::new(&remote_info);

                // Check if remote is already added
                let already_added = installation.remotes().contains_remote(&sk_remote);

                return Ok(SkSideloadable::new_repo(
                    file,
                    &sk_remote,
                    already_added,
                    installation,
                ));
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        task.await_result().await?;

        if let Some(dry_run) = task.result_dry_run() {
            Ok(SkSideloadable::new_package(
                file,
                &kind,
                &dry_run,
                installation,
            ))
        } else {
            // Never should happen (in theory)
            Err(Error::UnsupportedSideloadType)
        }
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new()
    }
}
