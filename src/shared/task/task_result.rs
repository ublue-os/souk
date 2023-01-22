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
use zbus::zvariant::{Optional, Type};

use crate::shared::dry_run::DryRun;
use crate::shared::WorkerError;

#[derive(Deserialize, Serialize, Type, PartialEq, Debug, Clone)]
pub struct TaskResult {
    pub type_: TaskResultType,
    pub dry_run: Optional<DryRun>,
    pub error: Optional<WorkerError>,
}

impl TaskResult {
    pub fn new_done() -> Self {
        Self {
            type_: TaskResultType::Done,
            dry_run: None.into(),
            error: None.into(),
        }
    }

    pub fn new_dry_run(dry_run: DryRun) -> Self {
        Self {
            type_: TaskResultType::DoneDryRun,
            dry_run: Some(dry_run).into(),
            error: None.into(),
        }
    }

    pub fn new_error(error: WorkerError) -> Self {
        Self {
            type_: TaskResultType::Error,
            dry_run: None.into(),
            error: Some(error).into(),
        }
    }

    pub fn new_cancelled() -> Self {
        Self {
            type_: TaskResultType::Cancelled,
            dry_run: None.into(),
            error: None.into(),
        }
    }
}

impl Default for TaskResult {
    fn default() -> Self {
        Self {
            type_: TaskResultType::None,
            dry_run: None.into(),
            error: None.into(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskResultType {
    /// Task completed successfully.
    Done,
    /// Task completed successfully, with an [DryRun] as result
    DoneDryRun,
    /// Task failed. See [ResponseType.error] for more details.
    Error,
    /// Task got cancelled (most likely by user).
    Cancelled,
    #[default]
    None,
}
