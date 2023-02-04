// Souk - flatpak_operation_type.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
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

use std::fmt;

use glib::Enum;
use gtk::glib;

use crate::main::i18n::i18n;
use crate::shared::flatpak::FlatpakOperationType;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkFlatpakOperationType")]
pub enum SkFlatpakOperationType {
    Install,
    InstallBundle,
    Uninstall,
    Update,
    None,
}

impl From<FlatpakOperationType> for SkFlatpakOperationType {
    fn from(op: FlatpakOperationType) -> Self {
        match op {
            FlatpakOperationType::Install => Self::Install,
            FlatpakOperationType::InstallBundle => Self::InstallBundle,
            FlatpakOperationType::Update => Self::Update,
            FlatpakOperationType::Uninstall => Self::Uninstall,
            FlatpakOperationType::None => Self::None,
        }
    }
}

impl fmt::Display for SkFlatpakOperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Install => i18n("Install"),
            Self::InstallBundle => i18n("Bundle Install"),
            Self::Update => i18n("Update"),
            Self::Uninstall => i18n("Uninstall"),
            Self::None => i18n("None"),
        };

        write!(f, "{text}")
    }
}

impl Default for SkFlatpakOperationType {
    fn default() -> Self {
        SkFlatpakOperationType::None
    }
}
