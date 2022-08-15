// Souk - dbus_server.rs
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

use async_std::channel::{unbounded, Receiver, Sender};
use async_std::prelude::*;
use uuid::Uuid;
use zbus::zvariant::Optional;
use zbus::{dbus_interface, ConnectionBuilder, SignalContext};

use crate::config;
use crate::worker::installation::{InstallationInfo, InstallationManager};
use crate::worker::transaction::{
    DryRunResult, TransactionCommand, TransactionError, TransactionMessage, TransactionProgress,
};
use crate::worker::WorkerError;

#[derive(Debug)]
struct Worker {
    installation_manager: InstallationManager,
    transaction_sender: Sender<TransactionCommand>,
}

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl Worker {
    // Transaction

    async fn install_flatpak(
        &self,
        ref_: &str,
        remote: &str,
        installation_id: &str,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.transaction_sender
            .send(TransactionCommand::InstallFlatpak(
                uuid.clone(),
                ref_.to_string(),
                remote.to_string(),
                installation_id.to_string(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_bundle(
        &self,
        path: &str,
        installation_id: &str,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.transaction_sender
            .send(TransactionCommand::InstallFlatpakBundle(
                uuid.clone(),
                path.to_string(),
                installation_id.to_string(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation_id: &str,
    ) -> Result<DryRunResult, WorkerError> {
        let (transaction_sender, mut receiver) = unbounded();

        self.transaction_sender
            .send(TransactionCommand::InstallFlatpakBundleDryRun(
                path.to_string(),
                installation_id.to_string(),
                transaction_sender,
            ))
            .await
            .unwrap();

        receiver.next().await.unwrap()
    }

    async fn install_flatpak_ref(
        &self,
        path: &str,
        installation_id: &str,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.transaction_sender
            .send(TransactionCommand::InstallFlatpakRef(
                uuid.clone(),
                path.to_string(),
                installation_id.to_string(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_ref_dry_run(
        &self,
        path: &str,
        installation_id: &str,
    ) -> Result<DryRunResult, WorkerError> {
        let (transaction_sender, mut receiver) = unbounded();

        self.transaction_sender
            .send(TransactionCommand::InstallFlatpakRefDryRun(
                path.to_string(),
                installation_id.to_string(),
                transaction_sender,
            ))
            .await
            .unwrap();

        receiver.next().await.unwrap()
    }

    async fn cancel_transaction(&self, transaction_uuid: &str) {
        self.transaction_sender
            .send(TransactionCommand::CancelTransaction(
                transaction_uuid.to_string(),
            ))
            .await
            .unwrap();
    }

    #[dbus_interface(signal)]
    async fn transaction_progress(
        signal_ctxt: &SignalContext<'_>,
        progress: TransactionProgress,
    ) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn transaction_error(
        signal_ctxt: &SignalContext<'_>,
        error: TransactionError,
    ) -> zbus::Result<()>;

    // Installation

    async fn launch_app(
        &self,
        installation_id: &str,
        ref_: &str,
        commit: &str,
    ) -> Result<(), WorkerError> {
        self.installation_manager
            .launch_app(installation_id, ref_, commit)
    }

    async fn installations(&self) -> Result<Vec<InstallationInfo>, WorkerError> {
        Ok(self.installation_manager.installations())
    }

    async fn installation_by_id(
        &self,
        installation_id: &str,
    ) -> Result<Optional<InstallationInfo>, WorkerError> {
        Ok(self
            .installation_manager
            .installation_by_id(installation_id)
            .into())
    }

    async fn preferred_installation(&self) -> Result<InstallationInfo, WorkerError> {
        self.installation_manager.preferred_installation()
    }

    async fn add_remote(&self, installation_id: &str, repo_path: &str) -> Result<(), WorkerError> {
        self.installation_manager
            .add_remote(installation_id, repo_path)
    }
}

pub async fn start(
    installation_manager: InstallationManager,
    transaction_sender: Sender<TransactionCommand>,
    mut receiver: Receiver<TransactionMessage>,
) -> zbus::Result<()> {
    let name = format!("{}.Worker", config::APP_ID);
    let path = "/de/haeckerfelix/Souk/Worker";
    let worker = Worker {
        transaction_sender,
        installation_manager,
    };

    let con = ConnectionBuilder::session()?
        .name(name)?
        .serve_at(path, worker)?
        .build()
        .await?;

    let signal_ctxt = SignalContext::new(&con, path).unwrap();
    while let Some(message) = receiver.next().await {
        match message {
            TransactionMessage::Progress(progress) => {
                // Emit `transaction_progress` signal via dbus
                Worker::transaction_progress(&signal_ctxt, progress)
                    .await
                    .unwrap()
            }
            TransactionMessage::Error(error) => {
                // Emit `transaction_error` signal via dbus
                Worker::transaction_error(&signal_ctxt, error)
                    .await
                    .unwrap()
            }
        }
    }
    debug!("Stopped.");

    Ok(())
}
