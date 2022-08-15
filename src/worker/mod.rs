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

/// Parsing appstream metadata, creation of xmlb exports
mod appstream;
/// Handling of Flatpak installations with its remotes
mod installation;
mod package;
/// Execution of Flatpak (dry-run) transactions, and state tracking
mod transaction;

mod dbus_server;
mod process;
mod worker_error;

use installation::InstallationManager;
pub use installation::{InstallationInfo, RemoteInfo};
pub use package::PackageInfo;
pub use process::Process;
use transaction::TransactionManager;
pub use transaction::{DryRunResult, TransactionError, TransactionProgress};
pub use worker_error::WorkerError;

/// Start DBus server. This method gets called from the `souk-worker` binary.
pub async fn spawn_dbus_server() {
    use async_std::channel::unbounded;
    debug!("Start souk-worker dbus server...");

    let installation_manager = InstallationManager::new();

    // The Flatpak transaction manager is multithreaded, because Flatpak
    // transactions are blocking. Therefore it uses message passing for inter
    // thread communication
    let (command_sender, command_receiver) = unbounded();
    let (message_sender, message_receiver) = unbounded();
    TransactionManager::start(
        installation_manager.clone(),
        message_sender,
        command_receiver,
    );

    dbus_server::start(installation_manager, command_sender, message_receiver)
        .await
        .unwrap();
}
