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

use futures::future::join;
use futures_util::stream::StreamExt;
use gio::{File, ListStore};
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::Ref;
use once_cell::sync::Lazy;

use crate::error::Error;
use crate::flatpak::sideload::{BundleSideloadable, SkSideloadType, SkSideloadable};
use crate::flatpak::{SkInstallation, SkTransaction, SkTransactionModel, SkTransactionType};
use crate::worker::{DryRunError, Process, WorkerProxy};

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
                        "Transactions",
                        "Transactions",
                        SkTransactionModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "installations",
                        "Installations",
                        "Installations",
                        ListStore::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "preferred-installation",
                        "Preferred Installation",
                        "Preferred Installation",
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
    pub async fn start_process(&self) {
        let imp = self.imp();

        // Start `souk-worker` process
        imp.process.spawn();

        // Wait process is ready
        while (self.imp().proxy.installations().await).is_err() {}

        // Retrieve default installations
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

    /// Stops the `souk-worker` process
    pub fn stop_process(&self) {
        self.imp().process.kill();
    }

    /// Install new Flatpak by ref name
    pub async fn install_flatpak(
        &self,
        ref_: &Ref,
        remote: &str,
        installation: &str,
    ) -> Result<SkTransaction, Error> {
        let ref_string = ref_.format_ref().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak(&ref_string, remote, installation)
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
    ) -> Result<SkTransaction, Error> {
        let path = file.path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let transaction_uuid = self
            .imp()
            .proxy
            .install_flatpak_bundle(&path_string, installation)
            .await?;

        let type_ = SkTransactionType::Install;
        let transaction = SkTransaction::new(
            &transaction_uuid,
            &ref_,
            &type_,
            filename_string,
            installation,
        );
        self.add_transaction(&transaction);

        Ok(transaction)
    }

    /// Cancel a Flatpak transaction
    pub async fn cancel_transaction(&self, uuid: &str) -> Result<(), Error> {
        self.imp().proxy.cancel_transaction(uuid).await?;
        Ok(())
    }

    /// Opens a sideloadable Flatpak file and load it into a `SkSideloadable`
    /// which can be viewed / installed in a `SkSideloadWindow`
    pub async fn load_sideloadable(
        &self,
        file: &File,
        installation_uuid: &str,
    ) -> Result<SkSideloadable, Error> {
        let proxy = &self.imp().proxy;
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let type_ = SkSideloadType::determine_type(&file);
        let dry_run_results = match type_ {
            SkSideloadType::Bundle => {
                proxy
                    .install_flatpak_bundle_dry_run(&path_string, installation_uuid)
                    .await
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        debug!("Dry run results: {:#?}", dry_run_results);
        let dry_run_results = match dry_run_results {
            Ok(sideloadable) => sideloadable,
            Err(err) => match err {
                DryRunError::RuntimeNotFound(runtime) => {
                    return Err(Error::DryRunRuntimeNotFound(runtime))
                }
                DryRunError::Other(message) => return Err(Error::DryRunError(message)),
                DryRunError::ZBus(err) => return Err(Error::DbusError(err)),
            },
        };

        let sideloadable = match type_ {
            SkSideloadType::Bundle => {
                BundleSideloadable::new(&file, dry_run_results, installation_uuid)
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        let sideloadable = SkSideloadable::new(Box::new(sideloadable));
        Ok(sideloadable)
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
