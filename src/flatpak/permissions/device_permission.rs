// Souk - device_permission.rs
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

#[glib::flags(name = "SkDevicePermission")]
pub enum SkDevicePermission {
    #[flags_value(name = "none")]
    NONE = 1 << 0,
    #[flags_value(name = "unknown")]
    UNKNOWN = 1 << 1,
    #[flags_value(name = "dri")]
    DRI = 1 << 2,
    #[flags_value(name = "kvm")]
    KVM = 1 << 3,
    #[flags_value(name = "shm")]
    SHM = 1 << 4,
    #[flags_value(name = "all")]
    ALL = 1 << 5,
}

impl From<&str> for SkDevicePermission {
    fn from(value: &str) -> Self {
        match value {
            "dri" => Self::DRI,
            "kvm" => Self::KVM,
            "shm" => Self::SHM,
            "all" => Self::ALL,
            _ => Self::UNKNOWN,
        }
    }
}
