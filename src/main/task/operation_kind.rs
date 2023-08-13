// Souk - operation_kind.rs
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

use crate::shared::appstream::AppstreamOperationKind;
use crate::shared::flatpak::FlatpakOperationKind;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkOperationKind")]
#[derive(Default)]
pub enum SkOperationKind {
    FlatpakInstall,
    FlatpakUninstall,
    FlatpakUpdate,
    AppstreamSync,
    AppstreamCompile,
    #[default]
    None,
}

impl From<FlatpakOperationKind> for SkOperationKind {
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

impl From<AppstreamOperationKind> for SkOperationKind {
    fn from(kind: AppstreamOperationKind) -> Self {
        match kind {
            AppstreamOperationKind::Sync => Self::AppstreamSync,
            AppstreamOperationKind::Compile => Self::AppstreamCompile,
            AppstreamOperationKind::None => Self::None,
        }
    }
}
