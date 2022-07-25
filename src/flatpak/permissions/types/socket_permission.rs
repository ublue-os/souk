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

use crate::flatpak::context::{SkContextDetail, SkContextDetailLevel, SkContextDetailType};
use crate::flatpak::permissions::{PermissionDetails, SkPermissionSummary};
use crate::i18n::i18n;

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

impl PermissionDetails for SkSocketPermission {
    fn summary(&self) -> SkPermissionSummary {
        let mut summary = SkPermissionSummary::empty();

        if self.contains(Self::X11) && !self.contains(Self::FALLBACK_X11) {
            summary |= SkPermissionSummary::READ_DATA;
        }

        if self.contains(Self::SESSION_BUS) {
            summary |= SkPermissionSummary::FULL_SESSION_BUS_ACCESS;
        }

        if self.contains(Self::SYSTEM_BUS) {
            summary |= SkPermissionSummary::FULL_SYSTEM_BUS_ACCESS;
        }

        if self.contains(Self::SSH_AUTH) {
            summary |= SkPermissionSummary::READ_DATA;
        }

        if self.contains(Self::PCSC) {
            summary |= SkPermissionSummary::READ_DATA;
        }

        if self.contains(Self::UNKNOWN) {
            summary |= SkPermissionSummary::UNKNOWN;
        }

        summary
    }

    fn context_details(&self) -> Vec<SkContextDetail> {
        let mut details = Vec::new();
        let type_ = SkContextDetailType::Icon;
        let level = SkContextDetailLevel::Bad;

        if self.contains(Self::X11) && !self.contains(Self::FALLBACK_X11) {
            let icon_name = "computer-symbolic";
            let title = i18n("Legacy Windowing System");
            let description = i18n("Uses a legacy windowing system, can access the contents of other windows and intercept keyboard inputs");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::SESSION_BUS) {
            let icon_name = "system-run-symbolic";
            let title = i18n("Access to All Session Services");
            let description = i18n("Has access to all session services");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::SYSTEM_BUS) {
            let icon_name = "system-run-symbolic";
            let title = i18n("Access to All System Services");
            let description = i18n("Has access to all system services");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::SSH_AUTH) {
            let icon_name = "dialog-password-symbolic";
            let title = i18n("Access to SSH Authentication");
            let description = i18n("Can access SSH keys / can perform authentications");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::PCSC) {
            let icon_name = "dialog-password-symbolic";
            let title = i18n("Access to Smartcards / Security Keys");
            let description = i18n("Can access smartcards and security devices");

            details.push(SkContextDetail::new(
                type_,
                icon_name,
                level,
                &title,
                &description,
            ));
        }

        if self.contains(Self::CUPS) {
            let icon_name = "printer-symbolic";
            let level = SkContextDetailLevel::Neutral;
            let title = i18n("Access to Printer Management");
            let description = i18n("Unrestricted access to printer management");

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
            let title = i18n("Access to Unknown Socket");
            let description = i18n("Can access an unknown socket");

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
