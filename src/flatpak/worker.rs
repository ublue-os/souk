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

use std::cell::RefCell;

use flatpak::prelude::*;
use flatpak::{Ref, Remote};
use futures::future::join;
use futures_util::stream::StreamExt;
use gio::{File, ListStore};
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use crate::error::Error;
use crate::flatpak::dbus_proxy::WorkerProxy;
use crate::flatpak::installation::{SkInstallation, SkRemote};
use crate::flatpak::sideload::{SkSideloadType, SkSideloadable};
use crate::flatpak::transaction::{SkTransaction, SkTransactionModel, SkTransactionType};
use crate::worker::{InstallationInfo, Process, RemoteInfo};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub transactions: SkTransactionModel,
        pub installations: ListStore,
        pub preferred_installation: RefCell<Option<SkInstallation>>,

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
                        ListStore::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "preferred-installation",
                        "",
                        "",
                        SkInstallation::static_type(),
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
                "preferred-installation" => obj.preferred_installation().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

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
    pub fn installations(&self) -> ListStore {
        self.imp().installations.clone()
    }

    /// Returns all available Flatpak installations
    pub fn preferred_installation(&self) -> SkInstallation {
        self.imp().preferred_installation.borrow().clone().unwrap()
    }

    /// Starts the `souk-worker` process
    /// TODO: Automatically start / stop the worker process on demand
    pub async fn start_process(&self) {
        let imp = self.imp();

        // Start `souk-worker` process
        imp.process.spawn();

        // Wait process is ready
        while (imp.proxy.installations().await).is_err() {}

        self.refresh_installations().await;
    }

    /// Stops the `souk-worker` process
    pub fn stop_process(&self) {
        self.imp().process.kill();
    }

    pub async fn launch_app(
        &self,
        installation_id: &str,
        ref_: &Ref,
        commit: &str,
    ) -> Result<(), Error> {
        let ref_string = ref_.format_ref().unwrap().to_string();

        self.imp()
            .proxy
            .launch_app(installation_id, &ref_string, commit)
            .await?;

        Ok(())
    }

    /// Install new Flatpak by ref name
    pub async fn install_flatpak(
        &self,
        ref_: &Ref,
        remote: &str,
        installation: &str,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let ref_string = ref_.format_ref().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak(&ref_string, remote, installation, no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(&transaction_uuid, ref_, &type_, remote, installation);
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Install new Flatpak by bundle file
    pub async fn install_flatpak_bundle(
        &self,
        ref_: &Ref,
        file: &File,
        installation: &str,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak_bundle(&path_string, installation, no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(
            &transaction_uuid,
            ref_,
            &type_,
            filename_string,
            installation,
        );
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Install new Flatpak by ref file
    pub async fn install_flatpak_ref(
        &self,
        ref_: &Ref,
        file: &File,
        installation: &str,
        no_update: bool,
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak_ref(&path_string, installation, no_update)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(
            &transaction_uuid,
            ref_,
            &type_,
            filename_string,
            installation,
        );
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Add a new remote by flatpakrepo file
    pub async fn add_remote(&self, file: &File, installation_id: &str) -> Result<(), Error> {
        let path = file.path().unwrap().into_os_string();
        self.imp()
            .proxy
            .add_remote(installation_id, path.to_str().unwrap())
            .await?;
        Ok(())
    }

    /// Opens a sideloadable Flatpak file and load it into a `SkSideloadable`
    /// which can be viewed / installed in a `SkSideloadWindow`
    pub async fn load_sideloadable(
        &self,
        file: &File,
        installation_id: &str,
    ) -> Result<SkSideloadable, Error> {
        let proxy = &self.imp().proxy;
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let type_ = SkSideloadType::determine_type(file);
        let dry_run_result = match type_ {
            SkSideloadType::Bundle => {
                proxy
                    .install_flatpak_bundle_dry_run(&path_string, installation_id)
                    .await?
            }
            SkSideloadType::Ref => {
                proxy
                    .install_flatpak_ref_dry_run(&path_string, installation_id)
                    .await?
            }
            SkSideloadType::Repo => {
                let bytes = file.load_bytes(gio::Cancellable::NONE)?.0;
                let flatpak_remote = Remote::from_file("remote", &bytes)?;

                let name = flatpak_remote.name().unwrap().to_string();
                let url = flatpak_remote.url().unwrap().to_string();

                // Check if remote is already added
                let installation: Option<InstallationInfo> =
                    proxy.installation_by_id(installation_id).await?.into();
                let remotes = installation
                    .expect("Unable to retrieve installation")
                    .remotes;
                let already_added = remotes
                    .iter()
                    .any(|r| r.name == name || r.repository_url == url);

                let remote_info = RemoteInfo::new(&flatpak_remote, &installation_id);
                let remote = SkRemote::new(&remote_info);

                return Ok(SkSideloadable::new_repo(
                    file,
                    &remote,
                    already_added,
                    installation_id,
                ));
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        debug!("Dry run results: {:#?}", dry_run_result);
        Ok(SkSideloadable::new_package(
            file,
            type_,
            dry_run_result,
            installation_id,
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

    /// Updates the available Flatpak installations with its remotes
    pub async fn refresh_installations(&self) {
        let imp = self.imp();
        imp.installations.remove_all();

        // Retrieve default installations / remotes
        // TODO: Add a way to update them during runtime
        let installations = imp.proxy.installations().await.unwrap();
        for info in installations {
            let installation = SkInstallation::new(&info);
            imp.installations.append(&installation);
        }

        // The preferred installation is the one with the most refs installed
        let preferred_info = imp.proxy.preferred_installation().await.unwrap();
        let installation = SkInstallation::new(&preferred_info);
        *imp.preferred_installation.borrow_mut() = Some(installation);
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
