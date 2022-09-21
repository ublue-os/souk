// Souk - transaction_command.rs
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

use crate::shared::info::InstallationInfo;
use crate::worker::transaction::DryRunResult;
use crate::worker::WorkerError;

#[derive(Debug, Clone)]
pub enum FlatpakTask {
    // task_uuid, ref_, remote, installation_id, no_update
    InstallFlatpak(String, String, String, InstallationInfo, bool),
    // task_uuid, path, installation_id, no_update
    InstallFlatpakBundle(String, String, InstallationInfo, bool),
    // path, installation_id, sender
    InstallFlatpakBundleDryRun(
        String,
        InstallationInfo,
        Sender<Result<DryRunResult, WorkerError>>,
    ),
    // task_uuid, path, installation_id, no_update
    InstallFlatpakRef(String, String, InstallationInfo, bool),
    // path, installation_id, sender
    InstallFlatpakRefDryRun(
        String,
        InstallationInfo,
        Sender<Result<DryRunResult, WorkerError>>,
    ),
    // task_uuid,
    CancelTransaction(String),
}
