// Souk - operation_status.rs
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

use std::fmt;

use glib::Enum;
use gtk::glib;

use crate::main::i18n::i18n;
use crate::shared::task::response::OperationStatus;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkOperationStatus")]
#[derive(Default)]
pub enum SkOperationStatus {
    Pending,
    Installing,
    InstallingBundle,
    Updating,
    Uninstalling,
    Processing,
    Done,
    #[default]
    None,
}

impl SkOperationStatus {
    pub fn has_no_detailed_progress(&self) -> bool {
        self == &Self::InstallingBundle
    }
}

impl From<OperationStatus> for SkOperationStatus {
    fn from(status: OperationStatus) -> Self {
        match status {
            OperationStatus::Pending => Self::Pending,
            OperationStatus::Installing => Self::Installing,
            OperationStatus::InstallingBundle => Self::InstallingBundle,
            OperationStatus::Updating => Self::Updating,
            OperationStatus::Uninstalling => Self::Uninstalling,
            OperationStatus::Processing => Self::Processing,
            OperationStatus::Done => Self::Done,
            OperationStatus::None => Self::None,
        }
    }
}

impl fmt::Display for SkOperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Pending => i18n("Pending…"),
            Self::Installing => i18n("Installing…"),
            Self::InstallingBundle => i18n("Installing Bundle…"),
            Self::Updating => i18n("Updating…"),
            Self::Uninstalling => i18n("Uninstalling…"),
            Self::Processing => i18n("Processing…"),
            Self::Done => String::new(),
            Self::None => i18n("Unknown"),
        };

        write!(f, "{text}")
    }
}
