// Souk - task_type.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkTaskType")]
pub enum SkTaskType {
    /// A Flatpak package (with all related refs) gets installed
    FlatpakInstall,
    /// A Flatpak package (with all related refs) gets uninstalled
    FlatpakUninstall,
    /// One or more Flatpak packages are getting updated
    FlatpakUpdate,
    /// Never should be get used (placeholder)
    None,
    // Appstream...,
}
