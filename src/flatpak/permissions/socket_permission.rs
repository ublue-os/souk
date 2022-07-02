// Souk - socket_permission.rs
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

#[glib::flags(name = "SkSocketPermission")]
pub enum SkSocketPermission {
    #[flags_value(name = "none")]
    NONE = 1 << 0,
    #[flags_value(name = "unknown")]
    UNKNOWN = 1 << 1,
    #[flags_value(name = "x11")]
    X11 = 1 << 2,
    #[flags_value(name = "wayland")]
    WAYLAND = 1 << 3,
    #[flags_value(name = "fallback-x11")]
    FALLBACK_X11 = 1 << 4,
    #[flags_value(name = "pulseaudio")]
    PULSEAUDIO = 1 << 5,
    #[flags_value(name = "system-bus")]
    SYSTEM_BUS = 1 << 6,
    #[flags_value(name = "session-bus")]
    SESSION_BUS = 1 << 7,
    #[flags_value(name = "ssh-auth")]
    SSH_AUTH = 1 << 8,
    #[flags_value(name = "pcsc")]
    PCSC = 1 << 9,
    #[flags_value(name = "cups")]
    CUPS = 1 << 10,
}

impl From<&str> for SkSocketPermission {
    fn from(value: &str) -> Self {
        match value {
            "x11" => Self::X11,
            "wayland" => Self::WAYLAND,
            "fallback-x11" => Self::FALLBACK_X11,
            "pulseaudio" => Self::PULSEAUDIO,
            "system-bus" => Self::SYSTEM_BUS,
            "session-bus" => Self::SESSION_BUS,
            "ssh-auth" => Self::SSH_AUTH,
            "pcsc" => Self::PCSC,
            "cups" => Self::CUPS,
            _ => Self::UNKNOWN,
        }
    }
}
