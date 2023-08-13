// Souk - appstream_task.rs
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
use uuid::Uuid;

use crate::shared::task::{Task, TaskKind};

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub struct AppstreamTask {
    pub uuid: String,
    pub kind: AppstreamTaskKind,
}

impl AppstreamTask {
    pub fn new(kind: AppstreamTaskKind) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            kind,
        }
    }
}

impl From<AppstreamTask> for Task {
    fn from(appstream_task: AppstreamTask) -> Self {
        Task {
            uuid: appstream_task.uuid.clone(),
            cancellable: false,
            kind: TaskKind::Appstream(Box::new(appstream_task)),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum AppstreamTaskKind {
    Ensure,
    Update,
    #[default]
    None,
}
