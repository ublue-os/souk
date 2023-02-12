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

use crate::main::context::{SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::permissions::{PermissionDetails, SkPermissionSummary};
use crate::main::i18n::i18n;

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

impl PermissionDetails for SkDevicePermission {
    fn summary(&self) -> SkPermissionSummary {
        let mut summary = SkPermissionSummary::empty();

        if self.contains(Self::ALL) {
            summary |= SkPermissionSummary::FULL_DEVICE_ACCESS;
        }

        if self.contains(Self::SHM) {
            summary |= SkPermissionSummary::READ_DATA;
        }

        if self.contains(Self::UNKNOWN) {
            summary |= SkPermissionSummary::UNKNOWN;
        }

        summary
    }

    fn context_details(&self) -> Vec<SkContextDetail> {
        let mut details = Vec::new();
        let type_ = SkContextDetailKind::Icon;
        let icon_name = "drive-harddisk-usb-symbolic";

        if self == &Self::NONE || self == &SkDevicePermission::DRI {
            let level = SkContextDetailLevel::Good;
            let title = i18n("No Device Access");
            let description = i18n("Has no access to connected devices");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::ALL) {
            let level = SkContextDetailLevel::Bad;
            let title = i18n("Access to All Devices");
            let description =
                i18n("Can access all connected devices, e.g. webcams or game controllers");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if !self.contains(Self::ALL) && self.contains(Self::KVM) {
            let icon_name = "computer-symbolic";
            let level = SkContextDetailLevel::Neutral;
            let title = i18n("Access to Virtualization Subsystem");
            let description = i18n("Can access and run virtual machines");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::SHM) {
            let icon_name = "folder-symbolic";
            let level = SkContextDetailLevel::Warning;
            let title = i18n("Access to Shared Memory");
            let description = i18n("Can access memory which is shared with other applications");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::UNKNOWN) {
            let icon_name = "dialog-question-symbolic";
            let level = SkContextDetailLevel::Bad;
            let title = i18n("Access to Unknown Device");
            let description = i18n("Can access an unknown device");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        details
    }
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
