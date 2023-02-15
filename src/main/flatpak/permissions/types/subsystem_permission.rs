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

use crate::main::context::{SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::permissions::{PermissionDetails, SkPermissionSummary};
use crate::main::i18n::i18n;

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

impl PermissionDetails for SkSubsystemPermission {
    fn summary(&self) -> SkPermissionSummary {
        let mut summary = SkPermissionSummary::empty();

        if self.contains(Self::NETWORK) {
            summary |= SkPermissionSummary::NETWORK_ACCESS;
        }

        if self.contains(Self::UNKNOWN) {
            summary |= SkPermissionSummary::UNKNOWN;
        }

        summary
    }

    fn context_details(&self) -> Vec<SkContextDetail> {
        let mut details = Vec::new();
        let icon_name = "network-wireless-symbolic";
        let kind = SkContextDetailKind::Icon;

        if self.contains(Self::NETWORK) {
            let level = SkContextDetailLevel::Neutral;
            let title = i18n("Network Access");
            let description = i18n("Can access the internet / local network");

            details.push(SkContextDetail::new(
                kind,
                icon_name,
                level,
                &title,
                &description,
            ));
        } else {
            let level = SkContextDetailLevel::Good;
            let title = i18n("No Network Access");
            let description = i18n("Cannot access the internet / local network");

            details.push(SkContextDetail::new(
                kind,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::UNKNOWN) {
            let icon_name = "dialog-question-symbolic";
            let level = SkContextDetailLevel::Bad;
            let title = i18n("Access to Unknown Subsystem");
            let description = i18n("Can access an unknown subsystem");

            details.push(SkContextDetail::new(
                kind,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        details
    }
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
