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

mod operation;
mod operation_kind;
mod operation_model;
mod operation_status;
#[allow(clippy::module_inception)]
mod task;
mod task_kind;
mod task_model;
mod task_status;

pub use operation::SkOperation;
pub use operation_kind::SkOperationKind;
pub use operation_model::SkOperationModel;
pub use operation_status::SkOperationStatus;
pub use task::SkTask;
pub use task_kind::SkTaskKind;
pub use task_model::SkTaskModel;
pub use task_status::SkTaskStatus;
