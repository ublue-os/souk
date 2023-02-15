// Souk - service_permission.rs
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

use glib::{ParamSpec, Properties};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use lazy_static::lazy_static;
use once_cell::unsync::OnceCell;

use crate::main::context::{SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::permissions::{PermissionDetails, SkPermissionSummary};
use crate::main::i18n::{i18n, i18n_f};

lazy_static! {
    static ref SENSITIVE_SERVICES: Vec<&'static str> = vec![
        "org.gnome.SessionManager",
        "org.freedesktop.PackageKit",
        "org.freedesktop.NetworkManager",
        "org.freedesktop.UDisks2",
        "ca.desrt.dconf",
        "org.gnome.SettingsDaemon",
        "org.freedesktop.secrets",
        "org.freedesktop.Flatpak"
    ];
}

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkServicePermission)]
    pub struct SkServicePermission {
        #[property(get, set, construct_only)]
        name: OnceCell<String>,
        #[property(get, set, construct_only)]
        is_system: OnceCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkServicePermission {
        const NAME: &'static str = "SkServicePermission";
        type Type = super::SkServicePermission;
    }

    impl ObjectImpl for SkServicePermission {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
    }
}

glib::wrapper! {
    pub struct SkServicePermission(ObjectSubclass<imp::SkServicePermission>);
}

impl SkServicePermission {
    pub fn new(name: &str, is_system: bool) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("is-system", is_system)
            .build()
    }

    pub fn no_access_context() -> SkContextDetail {
        let type_ = SkContextDetailKind::Icon;
        let icon_name = "system-run-symbolic".to_string();
        let level = SkContextDetailLevel::Good;

        let title = i18n("No Access to Services");
        let description = i18n("Does not communicate with any service");

        SkContextDetail::new(type_, &icon_name, level, &title, &description)
    }
}

impl PermissionDetails for SkServicePermission {
    fn summary(&self) -> SkPermissionSummary {
        let mut summary = SkPermissionSummary::empty();

        if SENSITIVE_SERVICES
            .iter()
            .any(|i| self.name().starts_with(i))
        {
            summary |= SkPermissionSummary::READWRITE_DATA;
        }

        if self.name().starts_with("org.freedesktop.Flatpak") {
            summary |= SkPermissionSummary::SANDBOX_ESCAPE;
        }

        summary
    }

    fn context_details(&self) -> Vec<SkContextDetail> {
        let type_ = SkContextDetailKind::Icon;
        let icon_name = "system-run-symbolic".to_string();
        let mut level = if !self.is_system() {
            SkContextDetailLevel::Neutral
        } else {
            SkContextDetailLevel::Moderate
        };
        let mut title = if !self.is_system() {
            i18n_f("Access to “{}” Service", &[&self.name()])
        } else {
            i18n_f("Access to System Service “{}”", &[&self.name()])
        };
        let mut description = i18n("Allows sending commands to the service and sharing data");

        // Well known dbus services

        if self.name().starts_with("org.gnome.SettingsDaemon") {
            title = i18n("Access to System Settings Service");
            description = i18n("Can read and modify system settings");
        }

        if self.name().starts_with("org.gnome.SessionManager") {
            title = i18n("Access to Session Manager Service");
            description = i18n("Has access to the current user session and open applications");
        }

        if self.name().starts_with("org.freedesktop.PackageKit") {
            title = i18n("Access to System Package Management");
            description = i18n("Can install, uninstall or update system packages");
        }

        if self.name().starts_with("org.freedesktop.Flatpak") {
            title = i18n("Access to Flatpak Service");
            description = i18n("Can execute arbitrary commands on the host system");
        }

        if self.name().starts_with("org.freedesktop.secrets") {
            title = i18n("Access to Password/Key Management Service");
            description = i18n("Can read, edit or delete passwords and keys");
        }

        if self.name().starts_with("org.freedesktop.FileManager") {
            title = i18n("Access to File Manager Service");
            description = i18n("Can open folders or files");
        }

        if self.name().starts_with("org.freedesktop.NetworkManager") {
            title = i18n("Access to System Network Service");
            description = i18n("Can read and modify network settings");
        }

        if self.name().starts_with("org.freedesktop.ScreenSaver") {
            title = i18n("Access to Screen Saver Service");
            description = i18n("Can read and modify screen saver settings");
        }

        if self.name().starts_with("org.freedesktop.Avahi") {
            title = i18n("Access to Avahi Network Service");
            description = i18n("Can publish or find information on the local network");
        }

        if self.name().starts_with("org.freedesktop.UPower") {
            title = i18n("Access to Power Management Service");
            description = i18n("Can read information about the current power consumption / status");
        }

        if self.name().starts_with("org.freedesktop.UDisks2") {
            title = i18n("Access to Disks Management Service");
            description = i18n("Can access, mount, unmount, or edit disk volumes");
        }

        if self.name().starts_with("ca.desrt.dconf") {
            title = i18n("Access to System Settings Database Service");
            description = i18n("Can read and modify system / application settings");
        }

        if SENSITIVE_SERVICES
            .iter()
            .any(|i| self.name().starts_with(i))
        {
            level = SkContextDetailLevel::Bad;
        }

        vec![SkContextDetail::new(
            type_,
            &icon_name,
            level,
            &title,
            &description,
        )]
    }
}
