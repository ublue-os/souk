// Souk - task_result.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use serde::{Deserialize, Serialize};

use crate::shared::flatpak::dry_run::DryRun;
use crate::shared::WorkerError;

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskResult {
    /// Task completed successfully.
    Done,
    /// Task completed successfully, with an [DryRun] as result
    DoneDryRun(Box<DryRun>),
    /// Task failed. See [ResponseType.error] for more details.
    Error(Box<WorkerError>),
    /// Task got cancelled (most likely by user).
    Cancelled,
    #[default]
    None,
}
