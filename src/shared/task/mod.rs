// Souk - mod.rs
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

mod appstream_task;
mod flatpak_task;
#[allow(clippy::module_inception)]
mod task;
mod task_progress;
mod task_response;
mod task_result;
mod task_status;

pub use appstream_task::AppstreamTask;
pub use flatpak_task::{FlatpakTask, FlatpakTaskType};
pub use task::Task;
pub use task_progress::TaskProgress;
pub use task_response::{TaskResponse, TaskResponseType};
pub use task_result::{TaskResult, TaskResultType};
pub use task_status::TaskStatus;
