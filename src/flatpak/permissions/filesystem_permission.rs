// Souk - filesystem_permission.rs
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

use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::SkFilesystemPermissionType;
use crate::flatpak::context::{SkContextDetail, SkContextDetailLevel, SkContextDetailType};
use crate::i18n::{i18n, i18n_f};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkFilesystemPermission {
        pub type_: OnceCell<SkFilesystemPermissionType>,
        pub path: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkFilesystemPermission {
        const NAME: &'static str = "SkFilesystemPermission";
        type ParentType = glib::Object;
        type Type = super::SkFilesystemPermission;
    }

    impl ObjectImpl for SkFilesystemPermission {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkFilesystemPermissionType::static_type(),
                        SkFilesystemPermissionType::ReadOnly as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("path", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "path" => obj.path().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkFilesystemPermission(ObjectSubclass<imp::SkFilesystemPermission>);
}

impl SkFilesystemPermission {
    pub fn new(value: &str) -> Self {
        let perm: Self = glib::Object::new(&[]).unwrap();
        let imp = perm.imp();

        let path: &str;
        let type_ = if value.ends_with(":rw") {
            path = value.trim_end_matches(":rw");
            SkFilesystemPermissionType::ReadWrite
        } else if value.ends_with(":create") {
            path = value.trim_end_matches(":create");
            SkFilesystemPermissionType::Create
        } else if value.ends_with(":ro") {
            path = value.trim_end_matches(":ro");
            SkFilesystemPermissionType::ReadOnly
        } else {
            path = value;
            SkFilesystemPermissionType::ReadWrite
        };

        imp.type_.set(type_).unwrap();
        imp.path.set(path.to_string()).unwrap();

        perm
    }

    pub fn type_(&self) -> SkFilesystemPermissionType {
        *self.imp().type_.get().unwrap()
    }

    pub fn path(&self) -> String {
        self.imp().path.get().unwrap().to_string()
    }

    pub fn to_context_detail(&self) -> SkContextDetail {
        let type_ = SkContextDetailType::Icon;
        let mut icon_name = "folder-documents-symbolic".to_string();
        let mut level = if self.type_() == SkFilesystemPermissionType::ReadOnly {
            SkContextDetailLevel::Moderate
        } else {
            SkContextDetailLevel::Warning
        };
        let mut title = if self.type_() == SkFilesystemPermissionType::ReadOnly {
            i18n_f("Read-Only Access to “{}”", &[&self.path()])
        } else {
            i18n_f("Read/Write Access to “{}”", &[&self.path()])
        };
        let mut description = if self.type_() == SkFilesystemPermissionType::ReadOnly {
            i18n("Can read data in the directory")
        } else {
            i18n("Can read and write data in the directory")
        };

        // host filesystem
        if self.path() == "host" {
            icon_name = "drive-harddisk-symbolic".into();
            level = SkContextDetailLevel::Bad;

            if self.type_() == SkFilesystemPermissionType::ReadOnly {
                title = i18n("Full File System Read/Write Access");
                description = i18n("Can read and write all data on the file system");
            } else {
                title = i18n("Full File System Read-Only Access");
                description = i18n("Can read all data on the file system");
            }
        }

        if self.path() == "host-os" {
            icon_name = "drive-harddisk-symbolic".into();
            level = SkContextDetailLevel::Bad;

            title = i18n("Full Access to System Libraries and Executables");
            description = i18n("Can access system libraries, executables and static data");
        }

        if self.path() == "host-etc" {
            icon_name = "drive-harddisk-symbolic".into();
            level = SkContextDetailLevel::Bad;

            title = i18n("Full Access to System Configuration");
            description = i18n("Can access system configuration data from “/etc”");
        }

        // home filesystem
        if self.path() == "home" {
            icon_name = "emblem-documents-symbolic".into();
            level = SkContextDetailLevel::Bad;

            if self.type_() == SkFilesystemPermissionType::ReadOnly {
                title = i18n("Home Folder Read/Write Access");
                description = i18n("Can read and write all data in your home directory");
            } else {
                title = i18n("Home Folder Read-Only Access");
                description = i18n("Can read all data in your home directory");
            }
        }

        // xdg paths
        if self.path().starts_with("xdg-") {
            let mut subdir = None;
            let xdg = if self.path().contains('/') {
                let path = self.path();
                let split = path.splitn(2, '/').collect::<Vec<&str>>();
                subdir = Some(split.last().unwrap().to_string());
                split.first().unwrap().to_string()
            } else {
                self.path()
            };

            let xdg_title = match xdg.as_str() {
                "xdg-desktop" => {
                    icon_name = "user-desktop-symbolic".into();
                    i18n("Desktop")
                }
                "xdg-documents" => {
                    icon_name = "emblem-documents-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Documents")
                }
                "xdg-download" => {
                    icon_name = "folder-download-symbolic".into();
                    i18n("Downloads")
                }
                "xdg-music" => {
                    icon_name = "folder-music-symbolic".into();
                    i18n("Music")
                }
                "xdg-pictures" => {
                    icon_name = "folder-pictures-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Pictures")
                }
                "xdg-public-share" => {
                    icon_name = "folder-publicshare-symbolic".into();
                    i18n("Public")
                }
                "xdg-videos" => {
                    icon_name = "folder-videos-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Videos")
                }
                "xdg-templates" => {
                    icon_name = "folder-templates-symbolic".into();
                    i18n("Templates")
                }
                "xdg-config" => {
                    icon_name = "emblem-system-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Application-Config")
                }
                "xdg-cache" => {
                    icon_name = "folder-symbolic".into();
                    i18n("Application-Cache")
                }
                "xdg-data" => {
                    icon_name = "folder-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Application-Data")
                }
                "xdg-run" => {
                    icon_name = "system-run-symbolic".into();
                    level = SkContextDetailLevel::Bad;
                    i18n("Runtime")
                }
                _ => xdg,
            };

            if self.type_() == SkFilesystemPermissionType::ReadOnly {
                title = i18n_f("{} Folder Read-Only Access", &[&xdg_title]);
                if let Some(subdir) = subdir {
                    description = i18n_f(
                        "Can read data in the “{}” subdirectory in the “{}” directory",
                        &[&subdir, &xdg_title],
                    );
                } else {
                    description = i18n_f("Can read all data in the “{}” directory", &[&xdg_title]);
                }
            } else {
                title = i18n_f("{} Folder Read/Write Access", &[&xdg_title]);
                if let Some(subdir) = subdir {
                    description = i18n_f(
                        "Can read and write data in the “{}” subdirectory in the “{}” directory",
                        &[&subdir, &xdg_title],
                    );
                } else {
                    description = i18n_f(
                        "Can read and write all data in the “{}” directory",
                        &[&xdg_title],
                    );
                }
            }

            if self.type_() != SkFilesystemPermissionType::ReadOnly
                && self.path() == "xdg-data/flatpak/overrides"
            {
                title = i18n("Explicit Access to Flatpak System Folder");
                description = i18n("Can set arbitrary permissions, or change the permissions of other applications");
                level = SkContextDetailLevel::Bad;
            }
        }

        SkContextDetail::new(type_, &icon_name, level, &title, &description)
    }
}
