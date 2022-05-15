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
use crate::worker::flatpak;
use crate::worker::flatpak::DryRunResults;

#[zbus::dbus_proxy(interface = "de.haeckerfelix.Souk.Worker1")]
trait Worker {
    fn install_flatpak(&self, ref_: &str, remote: &str, installation: &str) -> Result<String>;

    fn install_flatpak_bundle(&self, path: &str, installation: &str) -> Result<String>;

    fn install_flatpak_bundle_dry_run(&self, path: &str) -> Result<DryRunResults>;

    fn cancel_transaction(&self, uuid: &str) -> Result<()>;

    #[dbus_proxy(signal)]
    fn progress(&self, progress: flatpak::Progress) -> Result<()>;

    #[dbus_proxy(signal)]
    fn error(&self, error: flatpak::Error) -> Result<()>;
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
