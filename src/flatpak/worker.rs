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
use futures::future::join;
use futures_util::stream::StreamExt;
use gio::File;
use glib::{clone, KeyFile, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use crate::error::Error;
use crate::flatpak::dbus_proxy::WorkerProxy;
use crate::flatpak::installation::{SkInstallation, SkInstallationModel, SkRemote};
use crate::flatpak::sideload::{SkSideloadType, SkSideloadable};
use crate::flatpak::transaction::{SkTransaction, SkTransactionModel, SkTransactionType};
use crate::flatpak::utils;
use crate::shared::RemoteInfo;
use crate::worker::Process;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub transactions: SkTransactionModel,
        pub installations: SkInstallationModel,

        pub proxy: WorkerProxy<'static>,
        pub process: Process,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorker {
        const NAME: &'static str = "SkWorker";
        type ParentType = glib::Object;
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
                let progress = obj.receive_transaction_progress();
                let error = obj.receive_transaction_error();
                join(progress, error).await;
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

    /// Starts the `souk-worker` process
    /// TODO: Automatically start / stop the worker process on demand
    pub async fn start_process(&self) {
        let imp = self.imp();

        // Start `souk-worker` process
        imp.process.spawn();

        // Wait process is ready
        // TODO: Don't sleep here
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    /// Stops the `souk-worker` process
    pub fn stop_process(&self) {
        self.imp().process.kill();
    }

    /// Install new Flatpak by ref name
    pub async fn install_flatpak(
        &self,
        ref_: &Ref,
        remote: &str,
        installation: &SkInstallation,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let ref_string = ref_.format_ref().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak(&ref_string, remote, installation.info(), no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(&transaction_uuid, ref_, &type_, remote);
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Install new Flatpak by bundle file
    pub async fn install_flatpak_bundle(
        &self,
        ref_: &Ref,
        file: &File,
        installation: &SkInstallation,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak_bundle(&path_string, installation.info(), no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(&transaction_uuid, ref_, &type_, filename_string);
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Install new Flatpak by ref file
    pub async fn install_flatpak_ref(
        &self,
        ref_: &Ref,
        file: &File,
        installation: &SkInstallation,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak_ref(&path_string, installation.info(), no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(&transaction_uuid, ref_, &type_, filename_string);
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Opens a sideloadable Flatpak file and load it into a `SkSideloadable`
    /// which can be viewed / installed in a `SkSideloadWindow`
    pub async fn load_sideloadable(
        &self,
        file: &File,
        installation: &SkInstallation,
    ) -> Result<SkSideloadable, Error> {
        let proxy = &self.imp().proxy;
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let type_ = SkSideloadType::determine_type(file);
        let dry_run_result = match type_ {
            SkSideloadType::Bundle => {
                proxy
                    .install_flatpak_bundle_dry_run(&path_string, installation.info())
                    .await?
            }
            SkSideloadType::Ref => {
                proxy
                    .install_flatpak_ref_dry_run(&path_string, installation.info())
                    .await?
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

    fn add_transaction(&self, transaction: &SkTransaction) {
        // Remove finished transactions from model
        transaction.connect_local(
            "done",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let transaction: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&transaction);
                None
            }),
        );
        transaction.connect_local(
            "cancelled",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let transaction: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&transaction);
                None
            }),
        );
        transaction.connect_local(
            "error",
            false,
            clone!(@weak self as this => @default-return None, move |t|{
                let transaction: SkTransaction = t[0].get().unwrap();
                this.transactions().remove_transaction(&transaction);
                None
            }),
        );

        self.transactions().add_transaction(transaction);
    }

    /// Cancel a Flatpak transaction
    pub async fn cancel_transaction(&self, transaction_uuid: &str) -> Result<(), Error> {
        self.imp()
            .proxy
            .cancel_transaction(transaction_uuid)
            .await?;
        Ok(())
    }

    /// Handle incoming progress messages from worker process
    async fn receive_transaction_progress(&self) {
        let mut progress = self
            .imp()
            .proxy
            .receive_transaction_progress()
            .await
            .unwrap();

        while let Some(progress) = progress.next().await {
            let progress = progress.args().unwrap().progress;
            debug!("Transaction progress: {:#?}", progress);

            let uuid = progress.transaction_uuid.clone();

            match self.transactions().transaction(&uuid) {
                Some(transaction) => transaction.handle_progress(&progress),
                None => warn!("Received progress for unknown transaction!"),
            }
        }
    }

    /// Handle incoming error messages from worker process
    async fn receive_transaction_error(&self) {
        let mut error = self.imp().proxy.receive_transaction_error().await.unwrap();

        while let Some(error) = error.next().await {
            let error = error.args().unwrap().error;
            error!("Transaction error: {:#?}", error.message);

            let uuid = error.transaction_uuid.clone();

            match self.transactions().transaction(&uuid) {
                Some(transaction) => transaction.handle_error(&error),
                None => warn!("Received error for unknown transaction!"),
            }
        }
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
