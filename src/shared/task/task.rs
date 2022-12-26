// Souk - task.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
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
use zbus::zvariant::{Optional, Type};

use crate::shared::task::{AppstreamTask, FlatpakTask};

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub struct Task {
    /// Each task has a unique UUID that can be used for identification. This is
    /// required, for example, if a running task should be cancelled.
    pub uuid: String,
    /// Whether the task can be cancelled
    pub cancellable: bool,

    // This should have been an enum, unfortunately not supported by zbus / dbus
    flatpak_task: Optional<FlatpakTask>,
    appstream_task: Optional<AppstreamTask>,
}

impl Task {
    pub fn new_flatpak(task: FlatpakTask, cancellable: bool) -> Self {
        let uuid = Uuid::new_v4().to_string();
        Self {
            uuid,
            cancellable,
            flatpak_task: Some(task).into(),
            appstream_task: None.into(),
        }
    }

    pub fn new_appstream(task: AppstreamTask, cancellable: bool) -> Self {
        let uuid = Uuid::new_v4().to_string();
        Self {
            uuid,
            cancellable,
            flatpak_task: None.into(),
            appstream_task: Some(task).into(),
        }
    }

    /// Returns [FlatpakTask] if this is a Flatpak task.
    pub fn flatpak_task(&self) -> Option<FlatpakTask> {
        self.flatpak_task.clone().into()
    }

    /// Returns [AppstreamTask] if this is a Flatpak task
    pub fn appstream_task(&self) -> Option<AppstreamTask> {
        self.appstream_task.clone().into()
    }
}
