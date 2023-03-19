// Souk - task_kind.rs
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

use crate::shared::flatpak::FlatpakOperationKind;
use crate::shared::task::{FlatpakTaskKind, Task};

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkTaskKind")]
#[derive(Default)]
pub enum SkTaskKind {
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
    #[default]
    None,
    // Appstream...,
}

impl SkTaskKind {
    pub fn from_task_data(data: &Task) -> Self {
        if let Some(flatpak_task) = data.flatpak_task() {
            if flatpak_task.dry_run {
                return Self::FlatpakDryRun;
            } else {
                return flatpak_task.kind.into();
            }
        }

        Self::Unknown
    }
}

impl From<FlatpakTaskKind> for SkTaskKind {
    fn from(kind: FlatpakTaskKind) -> Self {
        match kind {
            FlatpakTaskKind::Install => Self::FlatpakInstall,
            FlatpakTaskKind::InstallRefFile => Self::FlatpakInstall,
            FlatpakTaskKind::InstallBundleFile => Self::FlatpakInstall,
            FlatpakTaskKind::Update => Self::FlatpakUpdate,
            FlatpakTaskKind::UpdateInstallation => Self::FlatpakUpdateInstallation,
            FlatpakTaskKind::Uninstall => Self::FlatpakUninstall,
            FlatpakTaskKind::None => Self::None,
        }
    }
}

impl From<FlatpakOperationKind> for SkTaskKind {
    fn from(kind: FlatpakOperationKind) -> Self {
        match kind {
            FlatpakOperationKind::Install => Self::FlatpakInstall,
            FlatpakOperationKind::InstallBundle => Self::FlatpakInstall,
            FlatpakOperationKind::Update => Self::FlatpakUpdate,
            FlatpakOperationKind::Uninstall => Self::FlatpakUninstall,
            FlatpakOperationKind::None => Self::None,
        }
    }
}
