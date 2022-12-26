// Souk - worker.rs
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

use flatpak::prelude::*;
use flatpak::{Ref, Remote};
use futures_util::stream::StreamExt;
use gio::File;
use glib::{clone, KeyFile, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use crate::main::dbus_proxy::WorkerProxy;
use crate::main::error::Error;
use crate::main::flatpak::installation::{SkInstallation, SkInstallationModel, SkRemote};
use crate::main::flatpak::sideload::{SkSideloadType, SkSideloadable};
use crate::main::flatpak::transaction::{SkTransaction, SkTransactionModel};
use crate::main::flatpak::utils;
use crate::shared::info::RemoteInfo;
use crate::shared::task::FlatpakTask;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub transactions: SkTransactionModel,
        pub installations: SkInstallationModel,

        pub proxy: WorkerProxy<'static>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorker {
        const NAME: &'static str = "SkWorker";
        type Type = super::SkWorker;
    }

    impl ObjectImpl for SkWorker {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "transactions",
                        "",
                        "",
                        SkTransactionModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "installations",
                        "",
                        "",
                        SkInstallationModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "transactions" => obj.transactions().to_value(),
                "installations" => obj.installations().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if let Err(err) = self.installations.refresh() {
                error!(
                    "Unable to refresh Flatpak installations: {}",
                    err.to_string()
                );
                // TODO: Expose this in UI
            }

            let fut = clone!(@weak obj => async move {
                obj.receive_task_response().await;
            });
            gtk_macros::spawn!(fut);
        }
    }
}

glib::wrapper! {
    pub struct SkWorker(ObjectSubclass<imp::SkWorker>);
}

impl SkWorker {
    /// Returns all current active Flatpak transactions
    pub fn transactions(&self) -> SkTransactionModel {
        self.imp().transactions.clone()
    }

    /// Returns all available Flatpak installations
    pub fn installations(&self) -> SkInstallationModel {
        self.imp().installations.clone()
    }

    /// Install new Flatpak by ref name
    pub async fn install_flatpak(
        &self,
        ref_: &Ref,
        remote: &SkRemote,
        installation: &SkInstallation,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTransaction, Error> {
        let ref_string = ref_.format_ref().unwrap().to_string();

        let task_data = FlatpakTask::new_install(
            &installation.info(),
            &remote.info(),
            &ref_string,
            uninstall_before_install,
            dry_run,
        );

        let task = SkTransaction::new(task_data);
        self.run_task(&task).await?;

        Ok(task)
    }

    /// Install new Flatpak by bundle file
    pub async fn install_flatpak_bundle_file(
        &self,
        file: &File,
        installation: &SkInstallation,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let task_data = FlatpakTask::new_install_bundle_file(
            &installation.info(),
            &path_string,
            uninstall_before_install,
            dry_run,
        );

        let task = SkTransaction::new(task_data);
        self.run_task(&task).await?;

        Ok(task)
    }

    /// Install new Flatpak by ref file
    pub async fn install_flatpak_ref_file(
        &self,
        file: &File,
        installation: &SkInstallation,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let task_data = FlatpakTask::new_install_ref_file(
            &installation.info(),
            &path_string,
            uninstall_before_install,
            dry_run,
        );

        let task = SkTransaction::new(task_data);
        self.run_task(&task).await?;

        Ok(task)
    }

    async fn run_task(&self, task: &SkTransaction) -> Result<(), Error> {
        // Remove finished tasks from model
        task.connect_local(
            "done",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let task: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&task);
                None
            }),
        );
        task.connect_local(
            "cancelled",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let task: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&task);
                None
            }),
        );
        task.connect_local(
            "error",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let task: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&task);
                None
            }),
        );

        self.transactions().add_transaction(task);
        self.imp().proxy.run_task(task.data()).await?;

        Ok(())
    }

    /// Cancel a worker task
    pub async fn cancel_task(&self, task: &SkTransaction) -> Result<(), Error> {
        self.imp().proxy.cancel_task(task.data()).await?;
        Ok(())
    }

    /// Handle incoming task responses from worker process
    async fn receive_task_response(&self) {
        let mut response = self.imp().proxy.receive_task_response().await.unwrap();

        while let Some(response) = response.next().await {
            let response = response.args().unwrap().response;
            debug!("Task response: {:#?}", response);

            let task_uuid = response.uuid.clone();
            match self.transactions().transaction(&task_uuid) {
                Some(task) => task.handle_response(&response),
                None => warn!("Received response for unknown task!"),
            }
        }
    }

    /// Opens a sideloadable Flatpak file and load it into a `SkSideloadable`
    /// which can be viewed / installed in a `SkSideloadWindow`
    pub async fn load_sideloadable(
        &self,
        file: &File,
        installation: &SkInstallation,
    ) -> Result<SkSideloadable, Error> {
        let type_ = SkSideloadType::determine_type(file);
        let dry_run_result = match type_ {
            SkSideloadType::Bundle => {
                let task = self
                    .install_flatpak_bundle_file(file, installation, false, true)
                    .await?;
                task.await_dry_run_result().await.unwrap()
            }
            SkSideloadType::Ref => {
                let task = self
                    .install_flatpak_ref_file(file, installation, false, true)
                    .await?;
                task.await_dry_run_result().await.unwrap()
            }
            SkSideloadType::Repo => {
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
                let mut remote_info = RemoteInfo::from(&flatpak_remote);
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

        debug!("Dry run results: {:#?}", dry_run_result);
        Ok(SkSideloadable::new_package(
            file,
            type_,
            dry_run_result,
            installation,
        ))
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
