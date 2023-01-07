// Souk - task_type.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::Enum;
use gtk::glib;

use crate::shared::task::{FlatpakOperationType, Task};

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkTaskType")]
pub enum SkTaskType {
    /// A Flatpak operation gets dry ran
    FlatpakDryRun,
    /// A Flatpak package (with all related refs) gets installed
    FlatpakInstall,
    /// A Flatpak package (with all related refs) gets uninstalled
    FlatpakUninstall,
    /// One single Flatpak package gets updated (with all related refs)
    FlatpakUpdate,
    /// A whole Flatpak installation gets updated
    FlatpakUpdateInstallation,
    /// Never should be get used (placeholder)
    Unknown,
    None,
    // Appstream...,
}

impl SkTaskType {
    pub fn from_task_data(data: &Task) -> Self {
        if let Some(flatpak_task) = data.flatpak_task() {
            if flatpak_task.dry_run {
                return Self::FlatpakDryRun;
            } else {
                return flatpak_task.operation_type.into();
            }
        }

        Self::Unknown
    }

    pub fn targets_single_package(&self) -> bool {
        self == &Self::FlatpakInstall
            || self == &Self::FlatpakUninstall
            || self == &Self::FlatpakUpdate
    }
}

impl From<FlatpakOperationType> for SkTaskType {
    fn from(op: FlatpakOperationType) -> Self {
        match op {
            FlatpakOperationType::Install => Self::FlatpakInstall,
            FlatpakOperationType::InstallRefFile => Self::FlatpakInstall,
            FlatpakOperationType::InstallBundleFile => Self::FlatpakInstall,
            FlatpakOperationType::Update => Self::FlatpakUpdate,
            FlatpakOperationType::UpdateInstallation => Self::FlatpakUpdateInstallation,
            FlatpakOperationType::Uninstall => Self::FlatpakUninstall,
            FlatpakOperationType::None => Self::None,
        }
    }
}

impl From<String> for SkTaskType {
    fn from(string: String) -> Self {
        match string.as_str() {
            "install" => Self::FlatpakInstall,
            "install-bundle" => Self::FlatpakInstall,
            "update" => Self::FlatpakUpdate,
            "uninstall" => Self::FlatpakUninstall,
            _ => {
                error!("Unable to parse string as SkTaskType: {}", string);
                Self::Unknown
            }
        }
    }
}

impl Default for SkTaskType {
    fn default() -> Self {
        SkTaskType::None
    }
}
