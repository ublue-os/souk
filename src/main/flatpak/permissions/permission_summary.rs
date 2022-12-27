// Souk - permission_summary.rs
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

use std::fmt::Write;

use gtk::glib;

use crate::main::context::{SkContextDetail, SkContextDetailLevel, SkContextDetailType};
use crate::main::i18n::i18n;

#[glib::flags(name = "SkPermissionSummary")]
pub enum SkPermissionSummary {
    #[flags_value(name = "full-device-access")]
    FULL_DEVICE_ACCESS = 1 << 1,
    #[flags_value(name = "full-session-bus-access")]
    FULL_SESSION_BUS_ACCESS = 1 << 2,
    #[flags_value(name = "full-system-bus-access")]
    FULL_SYSTEM_BUS_ACCESS = 1 << 3,
    #[flags_value(name = "read-data")]
    READ_DATA = 1 << 4,
    #[flags_value(name = "readwrite-data")]
    READWRITE_DATA = 1 << 5,
    #[flags_value(name = "network-access")]
    NETWORK_ACCESS = 1 << 6,
    #[flags_value(name = "sandbox-escape")]
    SANDBOX_ESCAPE = 1 << 7,

    #[flags_value(name = "unknown")]
    UNKNOWN = 1 << 8,
}

impl SkPermissionSummary {
    pub fn as_context_detail(&self) -> SkContextDetail {
        let type_ = SkContextDetailType::Icon;
        let type_value = "security-high-symbolic".to_string();
        let level = SkContextDetailLevel::Neutral;
        let title = i18n("Isolated with restricted permissions");
        let mut descriptions = Vec::new();

        if self.contains(Self::NETWORK_ACCESS) {
            descriptions.push(i18n("Is able to access the network."));
        }

        if self.contains(Self::FULL_DEVICE_ACCESS) {
            descriptions.push(i18n(
                "Has access to connected devices like game controllers or webcams.",
            ));
        }

        if self.contains(Self::FULL_SESSION_BUS_ACCESS)
            || self.contains(Self::FULL_SESSION_BUS_ACCESS)
        {
            descriptions.push(i18n("Has permissions to access other system services."));
        }

        if self.contains(Self::READWRITE_DATA) {
            descriptions.push(i18n(
                "Can access and modify defined data without asking for permission.",
            ));
        } else if self.contains(Self::READ_DATA) {
            descriptions.push(i18n(
                "Can access defined data without asking for permission.",
            ));
        }

        if self.contains(Self::UNKNOWN) {
            descriptions.push(i18n("Has an unknown permission."));
        }

        if self.contains(Self::SANDBOX_ESCAPE) {
            descriptions.push(i18n("<b>Explicitly bypasses security isolation, and can change its own permissions and those of other apps.</b>"));
        }

        if descriptions.is_empty() {
            descriptions.push(i18n("Has no special permissions."));
        }

        let mut description = String::new();
        for d in descriptions {
            write!(&mut description, "{d} ").unwrap();
        }

        SkContextDetail::new(type_, &type_value, level, &title, &description)
    }
}
