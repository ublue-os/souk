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

use async_std::channel::{unbounded, Sender};
use async_std::prelude::*;
use uuid::Uuid;
use zbus::{dbus_interface, SignalContext};

use crate::shared::info::InstallationInfo;
use crate::worker::transaction::{
    DryRunResult, FlatpakTask, TransactionError, TransactionProgress,
};
use crate::worker::WorkerError;

#[derive(Debug)]
pub struct WorkerServer {
    pub flatpak_task_sender: Sender<FlatpakTask>,
}

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl WorkerServer {
    // Transaction

    async fn install_flatpak(
        &self,
        ref_: &str,
        remote: &str,
        installation_info: InstallationInfo,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.flatpak_task_sender
            .send(FlatpakTask::InstallFlatpak(
                uuid.clone(),
                ref_.to_string(),
                remote.to_string(),
                installation_info.clone(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_bundle(
        &self,
        path: &str,
        installation_info: InstallationInfo,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.flatpak_task_sender
            .send(FlatpakTask::InstallFlatpakBundle(
                uuid.clone(),
                path.to_string(),
                installation_info.clone(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation_info: InstallationInfo,
    ) -> Result<DryRunResult, WorkerError> {
        let (flatpak_task_sender, mut receiver) = unbounded();

        self.flatpak_task_sender
            .send(FlatpakTask::InstallFlatpakBundleDryRun(
                path.to_string(),
                installation_info.clone(),
                flatpak_task_sender,
            ))
            .await
            .unwrap();

        receiver.next().await.unwrap()
    }

    async fn install_flatpak_ref(
        &self,
        path: &str,
        installation_info: InstallationInfo,
        no_update: bool,
    ) -> String {
        let uuid = Uuid::new_v4().to_string();
        self.flatpak_task_sender
            .send(FlatpakTask::InstallFlatpakRef(
                uuid.clone(),
                path.to_string(),
                installation_info.clone(),
                no_update,
            ))
            .await
            .unwrap();

        uuid
    }

    async fn install_flatpak_ref_dry_run(
        &self,
        path: &str,
        installation_info: InstallationInfo,
    ) -> Result<DryRunResult, WorkerError> {
        let (flatpak_task_sender, mut receiver) = unbounded();

        self.flatpak_task_sender
            .send(FlatpakTask::InstallFlatpakRefDryRun(
                path.to_string(),
                installation_info.clone(),
                flatpak_task_sender,
            ))
            .await
            .unwrap();

        receiver.next().await.unwrap()
    }

    async fn cancel_transaction(&self, transaction_uuid: &str) {
        self.flatpak_task_sender
            .send(FlatpakTask::CancelTransaction(transaction_uuid.to_string()))
            .await
            .unwrap();
    }

    #[dbus_interface(signal)]
    pub async fn transaction_progress(
        signal_ctxt: &SignalContext<'_>,
        progress: TransactionProgress,
    ) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn transaction_error(
        signal_ctxt: &SignalContext<'_>,
        error: TransactionError,
    ) -> zbus::Result<()>;
}
