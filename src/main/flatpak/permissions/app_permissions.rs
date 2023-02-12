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

use gio::ListStore;
use glib::{KeyFile, ParamFlags, ParamSpec, ParamSpecFlags, ParamSpecObject, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::types::*;

lazy_static! {
    static ref SERVICE_WHITELIST: Vec<&'static str> = vec![
        "org.kde.StatusNotifier",
        "org.mpris.MediaPlayer",
        "org.freedesktop.Notifications",
        "com.canonical.AppMenu.Registrar",
        "com.canonical.indicator.application",
        "com.canonical.Unity.LauncherEntry",
        "org.a11y.Bus"
    ];
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkAppPermissions {
        pub filesystems: OnceCell<ListStore>,
        pub services: OnceCell<ListStore>,

        pub devices: OnceCell<SkDevicePermission>,
        pub sockets: OnceCell<SkSocketPermission>,
        pub subsystems: OnceCell<SkSubsystemPermission>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkAppPermissions {
        const NAME: &'static str = "SkAppPermissions";
        type Type = super::SkAppPermissions;
    }

    impl ObjectImpl for SkAppPermissions {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "filesystems",
                        "",
                        "",
                        ListStore::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "services",
                        "",
                        "",
                        ListStore::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFlags::new(
                        "devices",
                        "",
                        "",
                        SkDevicePermission::static_type(),
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFlags::new(
                        "sockets",
                        "",
                        "",
                        SkSocketPermission::static_type(),
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecFlags::new(
                        "subsystems",
                        "",
                        "",
                        SkSubsystemPermission::static_type(),
                        0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "filesystems" => self.obj().filesystems().to_value(),
                "services" => self.obj().services().to_value(),
                "devices" => self.obj().devices().to_value(),
                "sockets" => self.obj().sockets().to_value(),
                "subsystems" => self.obj().subsystems().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkAppPermissions(ObjectSubclass<imp::SkAppPermissions>);
}

impl SkAppPermissions {
    pub fn from_metadata(keyfile: &KeyFile) -> Self {
        let permissions: Self = glib::Object::new();
        let imp = permissions.imp();

        let filesystems = ListStore::new(SkFilesystemPermission::static_type());
        if let Ok(filesystem_list) = keyfile.string_list("Context", "filesystems") {
            for filesystem in filesystem_list {
                let value = SkFilesystemPermission::new(filesystem.to_str());
                filesystems.append(&value);
            }
        }
        imp.filesystems.set(filesystems).unwrap();

        let services = ListStore::new(SkServicePermission::static_type());
        if let Ok(session_list) = keyfile.keys("Session Bus Policy") {
            for service in session_list {
                if Self::is_whitelisted(SERVICE_WHITELIST.to_vec(), service.to_str()) {
                    continue;
                }
                let value = SkServicePermission::new(service.to_str(), false);
                services.append(&value);
            }
        }
        if let Ok(system_list) = keyfile.keys("System Bus Policy") {
            for service in system_list {
                if Self::is_whitelisted(SERVICE_WHITELIST.to_vec(), service.to_str()) {
                    continue;
                }
                let value = SkServicePermission::new(service.to_str(), true);
                services.append(&value);
            }
        }
        imp.services.set(services).unwrap();

        let mut devices = SkDevicePermission::NONE;
        if let Ok(device_list) = keyfile.string_list("Context", "devices") {
            for device in device_list {
                devices |= device.to_str().into();
                devices.remove(SkDevicePermission::NONE);
            }
        }
        imp.devices.set(devices).unwrap();

        let mut sockets = SkSocketPermission::NONE;
        if let Ok(socket_list) = keyfile.string_list("Context", "sockets") {
            for socket in socket_list {
                sockets |= socket.to_str().into();
                sockets.remove(SkSocketPermission::NONE);
            }
        }
        imp.sockets.set(sockets).unwrap();

        let mut subsystems = SkSubsystemPermission::NONE;
        if let Ok(subsystem_list) = keyfile.string_list("Context", "shared") {
            for subsystem in subsystem_list {
                subsystems |= subsystem.to_str().into();
                subsystems.remove(SkSubsystemPermission::NONE);
            }
        }
        imp.subsystems.set(subsystems).unwrap();

        permissions
    }

    /// Compares with a different `SkAppPermissions` object, and returns the
    /// additional permissions which aren't in `self`
    pub fn additional_permissions(&self, other: &Self) -> Self {
        let devices = other.devices().difference(self.devices());
        let sockets = other.sockets().difference(self.sockets());
        let subsystems = other.subsystems().difference(self.subsystems());

        let filesystems = ListStore::new(SkFilesystemPermission::static_type());
        for filesystem in other.filesystems().snapshot() {
            let filesystem: SkFilesystemPermission = filesystem.downcast().unwrap();
            if !self.filesystems().snapshot().iter().any(|a| {
                let a: &SkFilesystemPermission = a.downcast_ref().unwrap();
                a.path() == filesystem.path() && a.type_() == filesystem.type_()
            }) {
                filesystems.append(&filesystem);
            }
        }

        let services = ListStore::new(SkServicePermission::static_type());
        for service in other.services().snapshot() {
            let service: SkServicePermission = service.downcast().unwrap();
            if !self.services().snapshot().iter().any(|a| {
                let a: &SkServicePermission = a.downcast_ref().unwrap();
                a.name() == service.name() && a.is_system() == service.is_system()
            }) {
                services.append(&service);
            }
        }

        let permissions: Self = glib::Object::new();

        let imp = permissions.imp();
        imp.filesystems.set(filesystems).unwrap();
        imp.services.set(services).unwrap();

        imp.devices.set(devices).unwrap();
        imp.sockets.set(sockets).unwrap();
        imp.subsystems.set(subsystems).unwrap();

        permissions
    }

    pub fn filesystems(&self) -> ListStore {
        self.imp().filesystems.get().unwrap().clone()
    }

    pub fn services(&self) -> ListStore {
        self.imp().services.get().unwrap().clone()
    }

    pub fn devices(&self) -> SkDevicePermission {
        *self.imp().devices.get().unwrap()
    }

    pub fn sockets(&self) -> SkSocketPermission {
        *self.imp().sockets.get().unwrap()
    }

    pub fn subsystems(&self) -> SkSubsystemPermission {
        *self.imp().subsystems.get().unwrap()
    }

    fn is_whitelisted(list: Vec<&str>, value: &str) -> bool {
        let res = list.iter().any(|i| value.starts_with(i));
        if res {
            debug!("Ignoring whitelisted permission entry: {}", value);
        }
        res
    }
}
