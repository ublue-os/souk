// Souk - service_permission.rs
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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, ToValue};
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::flatpak::context::{SkContextDetail, SkContextDetailLevel, SkContextDetailType};
use crate::i18n::{i18n, i18n_f};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkServicePermission {
        pub name: OnceCell<String>,
        pub is_system: OnceCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkServicePermission {
        const NAME: &'static str = "SkServicePermission";
        type ParentType = glib::Object;
        type Type = super::SkServicePermission;
    }

    impl ObjectImpl for SkServicePermission {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-system", "", "", false, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "is-system" => obj.is_system().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkServicePermission(ObjectSubclass<imp::SkServicePermission>);
}

impl SkServicePermission {
    pub fn new(name: &str, is_system: bool) -> Self {
        let perm: Self = glib::Object::new(&[]).unwrap();

        let imp = perm.imp();
        imp.name.set(name.to_string()).unwrap();
        imp.is_system.set(is_system).unwrap();

        perm
    }

    pub fn name(&self) -> String {
        self.imp().name.get().unwrap().to_string()
    }

    pub fn is_system(&self) -> bool {
        *self.imp().is_system.get().unwrap()
    }

    pub fn to_context_detail(&self) -> SkContextDetail {
        let type_ = SkContextDetailType::Icon;
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
        let mut description = i18n("Allows sending commands to the service or transferring data");

        // Well known dbus services

        if self.name().starts_with("org.gnome.SettingsDaemon") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to System Settings Service");
            description = i18n("Can read and modify system settings");
        }

        if self.name().starts_with("org.gnome.SessionManager") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to Session Manager Service");
            description = i18n("Has access to the current user session and open applications");
        }

        if self.name().starts_with("org.freedesktop.PackageKit") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to System Package Management");
            description = i18n("Can install, uninstall or update system packages");
        }

        if self.name().starts_with("org.freedesktop.Flatpak") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to Flatpak Service");
            description = i18n("Can execute arbitrary commands on the host system");
        }

        if self.name().starts_with("org.freedesktop.secrets") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to Password/Key Management Service");
            description = i18n("Can read, edit or delete passwords or keys");
        }

        if self.name().starts_with("org.freedesktop.FileManager") {
            title = i18n("Access to File Manager Service");
            description = i18n("Can open folders or files");
        }

        if self.name().starts_with("org.freedesktop.NetworkManager") {
            level = SkContextDetailLevel::Bad;
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
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to Disks Management Service");
            description = i18n("Can access, mount, unmount, or edit disk volumes");
        }

        if self.name().starts_with("ca.desrt.dconf") {
            level = SkContextDetailLevel::Bad;
            title = i18n("Access to System Settings Database Service");
            description = i18n("Can read and modify system / application settings");
        }

        SkContextDetail::new(type_, &icon_name, level, &title, &description)
    }
}
