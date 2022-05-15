// Souk - process.rs
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

mod dbus;
mod flatpak;
mod process;

pub use dbus::WorkerProxy;
pub use flatpak::{DryRunError, DryRunResults, Error, Progress, TransactionHandler};
pub use process::Process;

/// Start DBus server and Flatpak transaction handler.
/// This method gets called from the `souk-worker` binary.
pub async fn spawn_dbus_server() {
    use async_std::channel::unbounded;
    debug!("Start souk-worker dbus server...");

    let (server_tx, server_rx) = unbounded();
    let (flatak_tx, flatpak_rx) = unbounded();

    TransactionHandler::start(flatak_tx, server_rx);
    dbus::server::start(server_tx, flatpak_rx).await.unwrap();
}
