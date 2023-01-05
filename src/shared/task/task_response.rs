// Souk - task_progress.rs
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

use crate::shared::task::{TaskProgress, TaskResult};

#[derive(Deserialize, Serialize, Type, PartialEq, Debug, Clone)]
pub struct TaskResponse {
    /// The UUID of the corresponding task
    pub uuid: String,
    pub type_: TaskResponseType,

    // This should have been an enum, unfortunately not supported by zbus / dbus
    /// Initial response that provides information about this task and all
    /// related steps / subtasks
    pub initial_response: Optional<Vec<TaskProgress>>,
    pub update_response: Optional<TaskProgress>,
    pub result_response: Optional<TaskResult>,
}

impl TaskResponse {
    pub fn new_initial(uuid: String, steps: Vec<TaskProgress>) -> Self {
        Self {
            uuid,
            type_: TaskResponseType::Initial,
            initial_response: Some(steps).into(),
            update_response: None.into(),
            result_response: None.into(),
        }
    }

    pub fn new_update(uuid: String, step: TaskProgress) -> Self {
        Self {
            uuid,
            type_: TaskResponseType::Update,
            initial_response: None.into(),
            update_response: Some(step).into(),
            result_response: None.into(),
        }
    }

    pub fn new_result(uuid: String, result: TaskResult) -> Self {
        Self {
            uuid,
            type_: TaskResponseType::Result,
            initial_response: None.into(),
            update_response: None.into(),
            result_response: Some(result).into(),
        }
    }
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskResponseType {
    /// Initial (first) response of a task. This includes a detailed list of all
    /// steps, see [TaskResponse.initial_response].
    Initial,
    /// Update response of a task, contains updated information of a single
    /// step, see [TaskResponse.update_response].
    Update,
    /// Task ended. See [TaskResponse.result_response] for more details.
    Result,
}
