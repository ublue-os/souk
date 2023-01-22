// Souk - task_status.rs
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
use crate::shared::task::TaskStatus;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkTaskStatus")]
pub enum SkTaskStatus {
    None,
    Pending,
    Preparing,
    Installing,
    InstallingBundle,
    Uninstalling,
    Updating,
    Done,
    Cancelled,
    Error,
}

impl SkTaskStatus {
    pub fn is_completed(&self) -> bool {
        self == &Self::Done || self == &Self::Cancelled || self == &Self::Error
    }

    pub fn has_no_detailed_progress(&self) -> bool {
        self == &Self::InstallingBundle
    }
}

impl From<TaskStatus> for SkTaskStatus {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => Self::Pending,
            TaskStatus::Installing => Self::Installing,
            TaskStatus::InstallingBundle => Self::InstallingBundle,
            TaskStatus::Uninstalling => Self::Uninstalling,
            TaskStatus::Updating => Self::Updating,
            TaskStatus::Done => Self::Done,
            TaskStatus::None => Self::None,
        }
    }
}

impl fmt::Display for SkTaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::None => i18n("Unknown"),
            Self::Pending => i18n("Pending…"),
            Self::Preparing => i18n("Preparing…"),
            Self::Installing => i18n("Installing…"),
            Self::InstallingBundle => i18n("Installing Bundle…"),
            Self::Uninstalling => i18n("Uninstalling…"),
            Self::Updating => i18n("Updating…"),
            Self::Done => i18n("Done"),
            Self::Cancelled => i18n("Cancelled"),
            Self::Error => i18n("Error"),
        };

        write!(f, "{text}")
    }
}

impl Default for SkTaskStatus {
    fn default() -> Self {
        SkTaskStatus::None
    }
}
