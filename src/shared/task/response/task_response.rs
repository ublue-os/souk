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

use crate::shared::task::response::{TaskResult, TaskUpdate};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TaskResponse {
    /// The UUID of the corresponding task
    pub uuid: String,
    pub kind: TaskResponseKind,

    // This should have been an enum, unfortunately not supported by zbus / dbus
    /// Initial response that provides information about this task and all
    /// related steps / subtasks
    pub initial_response: Option<Vec<TaskUpdate>>,
    pub update_response: Option<TaskUpdate>,
    pub result_response: Option<TaskResult>,
}

impl TaskResponse {
    pub fn new_initial(uuid: String, steps: Vec<TaskUpdate>) -> Self {
        Self {
            uuid,
            kind: TaskResponseKind::Initial,
            initial_response: Some(steps),
            update_response: None,
            result_response: None,
        }
    }

    pub fn new_update(uuid: String, step: TaskUpdate) -> Self {
        Self {
            uuid,
            kind: TaskResponseKind::Update,
            initial_response: None,
            update_response: Some(step),
            result_response: None,
        }
    }

    pub fn new_result(uuid: String, result: TaskResult) -> Self {
        Self {
            uuid,
            kind: TaskResponseKind::Result,
            initial_response: None,
            update_response: None,
            result_response: Some(result),
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskResponseKind {
    /// Initial (first) response of a task. This includes a detailed list of all
    /// steps, see [TaskResponse.initial_response].
    Initial,
    /// Update response of a task, contains updated information of a single
    /// step, see [TaskResponse.update_response].
    Update,
    /// Task ended. See [TaskResponse.result_response] for more details.
    Result,
}
