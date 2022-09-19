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
/// Execution of Flatpak (dry-run) transactions, and state tracking
mod transaction;

mod dbus_server;
mod worker_error;

mod app;
pub use app::SkWorkerApplication;
use transaction::TransactionManager;
// TODO: Don't expose them, or move them into `shared`
pub use transaction::{DryRunResult, TransactionError, TransactionProgress};
pub use worker_error::WorkerError;
