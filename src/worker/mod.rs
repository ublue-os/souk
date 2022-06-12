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
mod error;
mod flatpak;
mod process;

pub use dbus::WorkerProxy;
pub use error::WorkerError;
pub use flatpak::installation::{InstallationInfo, InstallationManager};
pub use flatpak::transaction::{
    TransactionDryRun, TransactionError, TransactionManager, TransactionProgress,
};
pub use process::Process;

/// Start DBus server and Flatpak transaction manager.
/// This method gets called from the `souk-worker` binary.
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

    dbus::server::start(installation_manager, command_sender, message_receiver)
        .await
        .unwrap();
}
