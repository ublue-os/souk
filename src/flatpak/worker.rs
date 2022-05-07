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
use glib::{clone, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use zbus::Result;

use crate::flatpak::{SkTransaction, SkTransactionModel};
use crate::worker::WorkerProxy;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub transactions: SkTransactionModel,
        pub proxy: WorkerProxy<'static>,
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
            let fut = clone!(@strong obj => async move {
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

    pub async fn install_flatpak(
        &self,
        ref_: &str,
        remote: &str,
        installation: &str,
    ) -> Result<()> {
        self.imp()
            .proxy
            .install_flatpak(ref_, remote, installation)
            .await
    }

    pub async fn install_flatpak_bundle(&self, path: &str, installation: &str) -> Result<()> {
        self.imp()
            .proxy
            .install_flatpak_bundle(path, installation)
            .await
    }

    async fn receive_progress(&self) {
        let mut progress = self.imp().proxy.receive_progress().await.unwrap();
        while let Some(progress) = progress.next().await {
            let progress = progress.args().unwrap().progress;
            debug!("Transaction progress: {:#?}", progress);

            let uuid = progress.transaction_uuid.clone();

            match self.transactions().transaction(&uuid) {
                Some(transaction) => transaction.update(&progress),
                None => {
                    let transaction = SkTransaction::new(&uuid);
                    transaction.update(&progress);
                    self.transactions().add_transaction(&transaction);
                }
            }
        }
    }

    async fn receive_error(&self) {
        let mut error = self.imp().proxy.receive_error().await.unwrap();
        while let Some(error) = error.next().await {
            let error = error.args().unwrap().error;
            error!(
                "Transaction {} failed: {}",
                error.transaction_uuid, error.message
            );
        }
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
