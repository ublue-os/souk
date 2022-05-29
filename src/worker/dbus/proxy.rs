// Souk - client.rs
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

use zbus::Result;

use crate::config;
use crate::worker::flatpak::installation::InstallationInfo;
use crate::worker::flatpak::transaction;
use crate::worker::flatpak::transaction::{TransactionDryRun, TransactionDryRunError};

#[zbus::dbus_proxy(interface = "de.haeckerfelix.Souk.Worker1")]
trait Worker {
    // Transaction

    fn install_flatpak(
        &self,
        ref_: &str,
        remote: &str,
        installation_uuid: &str,
        no_update: bool,
    ) -> Result<String>;

    fn install_flatpak_bundle(
        &self,
        path: &str,
        installation_uuid: &str,
        no_update: bool,
    ) -> Result<String>;

    fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation: &str,
    ) -> std::result::Result<TransactionDryRun, TransactionDryRunError>;

    fn cancel_transaction(&self, uuid: &str) -> Result<()>;

    #[dbus_proxy(signal)]
    fn transaction_progress(&self, progress: transaction::TransactionProgress) -> Result<()>;

    #[dbus_proxy(signal)]
    fn transaction_error(&self, error: transaction::TransactionError) -> Result<()>;

    // Installation

    fn launch_app(&self, installation_uuid: &str, ref_: &str, commit: &str) -> Result<()>;

    fn installations(&self) -> Result<Vec<InstallationInfo>>;

    fn preferred_installation(&self) -> Result<InstallationInfo>;
}

impl Default for WorkerProxy<'static> {
    fn default() -> Self {
        let fut = async {
            let session = zbus::Connection::session().await?;
            let name = format!("{}.Worker", config::APP_ID);

            WorkerProxy::builder(&session)
                .destination(name)?
                .path("/de/haeckerfelix/Souk/Worker")?
                .build()
                .await
        };

        async_std::task::block_on(fut).unwrap()
    }
}
