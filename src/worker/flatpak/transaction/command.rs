// Souk - action.rs
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

use async_std::channel::Sender;

use crate::worker::flatpak::transaction::{DryRunError, DryRunResults};

#[derive(Debug, Clone)]
pub enum TransactionCommand {
    // uuid, ref_, remote, installation_uuid, no_update
    InstallFlatpak(String, String, String, String, bool),
    // uuid, path, installation_uuid, no_update
    InstallFlatpakBundle(String, String, String, bool),
    // path, installation_uuid, sender
    InstallFlatpakBundleDryRun(String, String, Sender<Result<DryRunResults, DryRunError>>),
    // uuid,
    CancelTransaction(String),
}
