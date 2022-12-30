// Souk - task_activity.rs
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

use glib::Enum;
use gtk::glib;
use strum_macros::{EnumString, ToString};

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum, EnumString, ToString)]
#[strum(serialize_all = "kebab-case")]
#[repr(u32)]
#[enum_type(name = "SkTaskActivity")]
pub enum SkTaskActivity {
    None,
    Pending,
    Preparing,
    Processing,
    Install,
    InstallBundle,
    Uninstall,
    Update,
    Done,
}

impl Default for SkTaskActivity {
    fn default() -> Self {
        SkTaskActivity::None
    }
}
