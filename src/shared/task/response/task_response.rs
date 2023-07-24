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

use crate::shared::task::response::{OperationActivity, TaskResult};
use crate::shared::task::Task;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TaskResponse {
    /// The [Task] to which this [TaskResponse] belongs
    pub task: Task,
    pub kind: TaskResponseKind,
}

impl TaskResponse {
    pub fn new_operation_activity(task: Task, activity: Vec<OperationActivity>) -> Self {
        Self {
            task,
            kind: TaskResponseKind::OperationActivity(Box::new(activity)),
        }
    }

    pub fn new_result(task: Task, result: TaskResult) -> Self {
        Self {
            task,
            kind: TaskResponseKind::Result(Box::new(result)),
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskResponseKind {
    OperationActivity(Box<Vec<OperationActivity>>),
    Result(Box<TaskResult>),
}
