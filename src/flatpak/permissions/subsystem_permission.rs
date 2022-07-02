// Souk - subsystem_permission.rs
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

use gtk::glib;

#[glib::flags(name = "SkSubsystemPermission")]
pub enum SkSubsystemPermission {
    #[flags_value(name = "none")]
    NONE = 1 << 0,
    #[flags_value(name = "unknown")]
    UNKNOWN = 1 << 1,
    #[flags_value(name = "network")]
    NETWORK = 1 << 2,
    #[flags_value(name = "ipc")]
    IPC = 1 << 3,
}

impl From<&str> for SkSubsystemPermission {
    fn from(value: &str) -> Self {
        match value {
            "network" => Self::NETWORK,
            "ipc" => Self::IPC,
            _ => Self::UNKNOWN,
        }
    }
}
