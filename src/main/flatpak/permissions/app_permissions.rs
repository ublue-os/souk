// Souk - app_permissions.rs
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

use std::cell::OnceCell;
use std::sync::LazyLock;

use gio::ListStore;
use glib::{KeyFile, ParamSpec, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use super::types::*;

static SERVICE_WHITELIST: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "org.kde.StatusNotifier",
        "org.mpris.MediaPlayer",
        "org.freedesktop.Notifications",
        "com.canonical.AppMenu.Registrar",
        "com.canonical.indicator.application",
        "com.canonical.Unity.LauncherEntry",
        "org.a11y.Bus",
    ]
});

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkAppPermissions)]
    pub struct SkAppPermissions {
        #[property(get, set, construct_only)]
        filesystems: OnceCell<ListStore>,
        #[property(get, set, construct_only)]
        services: OnceCell<ListStore>,
        #[property(get, set, construct_only)]
        devices: OnceCell<SkDevicePermission>,
        #[property(get, set, construct_only)]
        sockets: OnceCell<SkSocketPermission>,
        #[property(get, set, construct_only)]
        subsystems: OnceCell<SkSubsystemPermission>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkAppPermissions {
        const NAME: &'static str = "SkAppPermissions";
        type Type = super::SkAppPermissions;
    }

    impl ObjectImpl for SkAppPermissions {
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

    impl SkAppPermissions {
        pub fn is_whitelisted(list: Vec<&str>, value: &str) -> bool {
            let res = list.iter().any(|i| value.starts_with(i));
            if res {
                debug!("Ignoring whitelisted permission entry: {}", value);
            }
            res
        }
    }
}

glib::wrapper! {
    pub struct SkAppPermissions(ObjectSubclass<imp::SkAppPermissions>);
}

impl SkAppPermissions {
    pub fn new(
        filesystems: &ListStore,
        services: &ListStore,
        devices: &SkDevicePermission,
        sockets: &SkSocketPermission,
        subsystems: &SkSubsystemPermission,
    ) -> Self {
        glib::Object::builder()
            .property("filesystems", filesystems)
            .property("services", services)
            .property("devices", devices)
            .property("sockets", sockets)
            .property("subsystems", subsystems)
            .build()
    }

    pub fn from_metadata(keyfile: &KeyFile) -> Self {
        let filesystems = ListStore::new::<SkFilesystemPermission>();
        if let Ok(filesystem_list) = keyfile.string_list("Context", "filesystems") {
            for filesystem in filesystem_list {
                let value = SkFilesystemPermission::from_flatpak(filesystem.to_str());
                filesystems.append(&value);
            }
        }

        let services = ListStore::new::<SkServicePermission>();
        if let Ok(session_list) = keyfile.keys("Session Bus Policy") {
            for service in session_list {
                if imp::SkAppPermissions::is_whitelisted(
                    SERVICE_WHITELIST.to_vec(),
                    service.to_str(),
                ) {
                    continue;
                }
                let value = SkServicePermission::new(service.to_str(), false);
                services.append(&value);
            }
        }
        if let Ok(system_list) = keyfile.keys("System Bus Policy") {
            for service in system_list {
                if imp::SkAppPermissions::is_whitelisted(
                    SERVICE_WHITELIST.to_vec(),
                    service.to_str(),
                ) {
                    continue;
                }
                let value = SkServicePermission::new(service.to_str(), true);
                services.append(&value);
            }
        }

        let mut devices = SkDevicePermission::NONE;
        if let Ok(device_list) = keyfile.string_list("Context", "devices") {
            for device in device_list {
                devices |= device.to_str().into();
                devices.remove(SkDevicePermission::NONE);
            }
        }

        let mut sockets = SkSocketPermission::NONE;
        if let Ok(socket_list) = keyfile.string_list("Context", "sockets") {
            for socket in socket_list {
                sockets |= socket.to_str().into();
                sockets.remove(SkSocketPermission::NONE);
            }
        }

        let mut subsystems = SkSubsystemPermission::NONE;
        if let Ok(subsystem_list) = keyfile.string_list("Context", "shared") {
            for subsystem in subsystem_list {
                subsystems |= subsystem.to_str().into();
                subsystems.remove(SkSubsystemPermission::NONE);
            }
        }

        Self::new(&filesystems, &services, &devices, &sockets, &subsystems)
    }

    /// Compares with a different `SkAppPermissions` object, and returns the
    /// additional permissions which aren't in `self`
    pub fn additional_permissions(&self, other: &Self) -> Self {
        let devices = other.devices().difference(self.devices());
        let sockets = other.sockets().difference(self.sockets());
        let subsystems = other.subsystems().difference(self.subsystems());

        let filesystems = ListStore::new::<SkFilesystemPermission>();
        for filesystem in other.filesystems().snapshot() {
            let filesystem: SkFilesystemPermission = filesystem.downcast().unwrap();
            if !self.filesystems().snapshot().iter().any(|a| {
                let a: &SkFilesystemPermission = a.downcast_ref().unwrap();
                a.path() == filesystem.path() && a.kind() == filesystem.kind()
            }) {
                filesystems.append(&filesystem);
            }
        }

        let services = ListStore::new::<SkServicePermission>();
        for service in other.services().snapshot() {
            let service: SkServicePermission = service.downcast().unwrap();
            if !self.services().snapshot().iter().any(|a| {
                let a: &SkServicePermission = a.downcast_ref().unwrap();
                a.name() == service.name() && a.is_system() == service.is_system()
            }) {
                services.append(&service);
            }
        }

        Self::new(&filesystems, &services, &devices, &sockets, &subsystems)
    }
}
