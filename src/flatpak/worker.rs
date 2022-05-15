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

use futures::future::join;
use futures_util::stream::StreamExt;
use gio::File;
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{BundleRef, Ref};
use once_cell::sync::Lazy;

use crate::error::Error;
use crate::flatpak::sideload::{Sideloadable, SkBundle, SkSideloadType};
use crate::flatpak::{SkTransaction, SkTransactionModel, SkTransactionType};
use crate::worker::{DryRunError, Process, WorkerProxy};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub transactions: SkTransactionModel,
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
                vec![ParamSpecObject::new(
                    "transactions",
                    "Transactions",
                    "Transactions",
                    SkTransactionModel::static_type(),
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "transactions" => obj.transactions().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let fut = clone!(@weak obj => async move {
                let progress = obj.receive_progress();
                let error = obj.receive_error();
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
    pub fn transactions(&self) -> SkTransactionModel {
        self.imp().transactions.clone()
    }

    pub fn start_process(&self) {
        self.imp().process.spawn();
    }

    pub fn stop_process(&self) {
        self.imp().process.kill();
    }

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

    pub async fn install_flatpak_bundle(
        &self,
        ref_: &BundleRef,
        installation: &str,
    ) -> Result<SkTransaction, Error> {
        let path = ref_.file().unwrap().path().unwrap();
        let filename_string = path.file_name().unwrap().to_str().unwrap();
        let path_string = path.to_str().unwrap().to_string();
        let ref_: Ref = ref_.clone().upcast();

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

    pub async fn load_sideloadable(
        &self,
        file: &File,
        type_: &SkSideloadType,
        installation: &str,
    ) -> Result<impl Sideloadable, Error> {
        let proxy = &self.imp().proxy;
        let path = file.path().unwrap();
        let path_string = path.to_str().unwrap().to_string();

        let dry_run = match type_ {
            SkSideloadType::Bundle => {
                proxy
                    .install_flatpak_bundle_dry_run(&path_string, installation)
                    .await
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        debug!("Dry run results: {:#?}", dry_run);
        let dry_run = match dry_run {
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
                let bundle = BundleRef::new(file).unwrap();
                SkBundle::new(&bundle, dry_run.download_size(), dry_run.installed_size())
            }
            _ => return Err(Error::UnsupportedSideloadType),
        };

        Ok(sideloadable)
    }

    pub async fn cancel_transaction(&self, uuid: &str) -> Result<(), Error> {
        self.imp().proxy.cancel_transaction(uuid).await?;
        Ok(())
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

    async fn receive_progress(&self) {
        let mut progress = self.imp().proxy.receive_progress().await.unwrap();
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

    async fn receive_error(&self) {
        let mut error = self.imp().proxy.receive_error().await.unwrap();
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
