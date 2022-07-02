// Souk - app_permissions.rs
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

use gio::ListStore;
use glib::{
    KeyFile, KeyFileFlags, ParamFlags, ParamSpec, ParamSpecFlags, ParamSpecObject, ToValue,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::*;

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
        type ParentType = glib::Object;
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

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "filesystems" => obj.filesystems().to_value(),
                "services" => obj.services().to_value(),
                "devices" => obj.devices().to_value(),
                "sockets" => obj.sockets().to_value(),
                "subsystems" => obj.subsystems().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkAppPermissions(ObjectSubclass<imp::SkAppPermissions>);
}

impl SkAppPermissions {
    pub fn from_metainfo(metainfo: &str) -> Self {
        let permissions: Self = glib::Object::new(&[]).unwrap();
        let imp = permissions.imp();

        let keyfile = KeyFile::new();
        keyfile
            .load_from_data(metainfo, KeyFileFlags::NONE)
            .unwrap();

        let filesystems = ListStore::new(SkFilesystemPermission::static_type());
        let filesystem_list = keyfile.string_list("Context", "filesystems").unwrap();
        for filesystem in filesystem_list {
            let value = SkFilesystemPermission::new(&filesystem);
            filesystems.append(&value);
        }
        imp.filesystems.set(filesystems).unwrap();

        let services = ListStore::new(SkServicePermission::static_type());
        let session_list = keyfile.keys("Session Bus Policy").unwrap().0;
        let system_list = keyfile.keys("System Bus Policy").unwrap().0;
        for service in session_list {
            let value = SkServicePermission::new(&service, false);
            services.append(&value);
        }
        for service in system_list {
            let value = SkServicePermission::new(&service, true);
            services.append(&value);
        }
        imp.services.set(services).unwrap();

        let mut devices = SkDevicePermission::NONE;
        let device_list = keyfile.string_list("Context", "devices").unwrap();
        for device in device_list {
            devices |= device.as_str().into();
            devices.remove(SkDevicePermission::NONE);
        }
        imp.devices.set(devices).unwrap();

        let mut sockets = SkSocketPermission::NONE;
        let socket_list = keyfile.string_list("Context", "sockets").unwrap();
        for socket in socket_list {
            sockets |= socket.as_str().into();
            sockets.remove(SkSocketPermission::NONE);
        }
        imp.sockets.set(sockets).unwrap();

        let mut subsystems = SkSubsystemPermission::NONE;
        let subsystem_list = keyfile.string_list("Context", "shared").unwrap();
        for subsystem in subsystem_list {
            subsystems |= subsystem.as_str().into();
            subsystems.remove(SkSubsystemPermission::NONE);
        }
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

    pub fn sockets(&self) -> SkDevicePermission {
        *self.imp().devices.get().unwrap()
    }

    pub fn subsystems(&self) -> SkDevicePermission {
        *self.imp().devices.get().unwrap()
    }
}
