// Souk - mod.rs
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

mod device_permission;
mod filesystem_permission;
mod filesystem_permission_kind;
mod service_permission;
mod socket_permission;
mod subsystem_permission;

pub use device_permission::SkDevicePermission;
pub use filesystem_permission::SkFilesystemPermission;
pub use filesystem_permission_kind::SkFilesystemPermissionKind;
pub use service_permission::SkServicePermission;
pub use socket_permission::SkSocketPermission;
pub use subsystem_permission::SkSubsystemPermission;
