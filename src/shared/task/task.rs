// Souk - task.rs
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

use gtk::glib;
use serde::{Deserialize, Serialize};

use crate::shared::task::{AppstreamTask, FlatpakTask};

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash, glib::Boxed)]
#[boxed_type(name = "Task", nullable)]
pub struct Task {
    /// Each task has a unique UUID that can be used for identification. This is
    /// required, for example, if a running task should be cancelled.
    pub uuid: String,
    /// Whether the task can be cancelled
    pub cancellable: bool,

    // TODO: This should have been an enum, unfortunately not supported by zbus / dbus
    pub flatpak_task: Option<FlatpakTask>,
    pub appstream_task: Option<AppstreamTask>,
}

impl Task {
    /// Returns [FlatpakTask] if this is a Flatpak task.
    pub fn flatpak_task(&self) -> Option<FlatpakTask> {
        self.flatpak_task.clone()
    }

    /// Returns [AppstreamTask] if this is a Flatpak task
    pub fn appstream_task(&self) -> Option<AppstreamTask> {
        self.appstream_task.clone()
    }
}
