// Souk - filesystem_permission.rs
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
use once_cell::unsync::OnceCell;

use super::SkFilesystemPermissionKind;
use crate::main::context::{SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::permissions::{PermissionDetails, SkPermissionSummary};
use crate::main::i18n::{i18n, i18n_f};

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkFilesystemPermission)]
    pub struct SkFilesystemPermission {
        #[property(
            get,
            set,
            construct_only,
            builder(SkFilesystemPermissionKind::ReadWrite)
        )]
        kind: OnceCell<SkFilesystemPermissionKind>,
        #[property(get, set, construct_only)]
        path: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkFilesystemPermission {
        const NAME: &'static str = "SkFilesystemPermission";
        type Type = super::SkFilesystemPermission;
    }

    impl ObjectImpl for SkFilesystemPermission {
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
    pub struct SkFilesystemPermission(ObjectSubclass<imp::SkFilesystemPermission>);
}

impl SkFilesystemPermission {
    pub fn new(kind: &SkFilesystemPermissionKind, path: &str) -> Self {
        glib::Object::builder()
            .property("kind", kind)
            .property("path", path)
            .build()
    }

    pub fn from_flatpak(value: &str) -> Self {
        let path: &str;
        let kind = if value.ends_with(":rw") {
            path = value.trim_end_matches(":rw");
            SkFilesystemPermissionKind::ReadWrite
        } else if value.ends_with(":create") {
            path = value.trim_end_matches(":create");
            SkFilesystemPermissionKind::Create
        } else if value.ends_with(":ro") {
            path = value.trim_end_matches(":ro");
            SkFilesystemPermissionKind::ReadOnly
        } else {
            path = value;
            SkFilesystemPermissionKind::ReadWrite
        };

        Self::new(&kind, path)
    }

    pub fn no_access_context() -> SkContextDetail {
        let type_ = SkContextDetailKind::Icon;
        let icon_name = "folder-documents-symbolic".to_string();
        let level = SkContextDetailLevel::Good;

        let title = i18n("No Filessystem Access");
        let description = i18n("Cannot access the filesystem at all");

        SkContextDetail::new(type_, &icon_name, level, &title, &description)
    }
}

impl PermissionDetails for SkFilesystemPermission {
    fn summary(&self) -> SkPermissionSummary {
        let mut summary = SkPermissionSummary::empty();

        let s = if self.kind() == SkFilesystemPermissionKind::ReadOnly {
            SkPermissionSummary::READ_DATA
        } else {
            SkPermissionSummary::READWRITE_DATA
        };
        summary |= s;

        if self.kind() != SkFilesystemPermissionKind::ReadOnly
            && self.path().contains("flatpak/overrides")
        {
            summary |= SkPermissionSummary::SANDBOX_ESCAPE;
        }

        summary
    }

    fn context_details(&self) -> Vec<SkContextDetail> {
        let path = if self.path().starts_with("~/") {
            self.path().replace("~/", "home/")
        } else {
            self.path()
        };

        let mut subdir = None;
        let permission = if path.contains('/') && !path.starts_with('/') {
            let p = path.clone();
            let split = p.splitn(2, '/').collect::<Vec<&str>>();
            subdir = Some(split.last().unwrap().to_string());
            split.first().unwrap().to_string()
        } else {
            path.clone()
        };

        let kind = SkContextDetailKind::Icon;
        let mut icon_name = "folder-documents-symbolic".to_string();
        let mut level = if self.kind() == SkFilesystemPermissionKind::ReadOnly {
            SkContextDetailLevel::Moderate
        } else {
            SkContextDetailLevel::Warning
        };

        let permission_object;
        let mut permission_title = None;
        let mut permission_description = None;
        let mut is_folder = true;

        match permission.as_str() {
            "home" => {
                permission_object = i18n("Home");
                icon_name = "user-home-symbolic".into();
                level = SkContextDetailLevel::Bad;

                if subdir.is_none() {
                    if self.kind() == SkFilesystemPermissionKind::ReadOnly {
                        permission_title = Some(i18n("Home Folder Read/Write Access"));
                        permission_description =
                            Some(i18n("Can read and write all data in your home directory"));
                    } else {
                        permission_title = Some(i18n("Home Folder Read-Only Access"));
                        permission_description =
                            Some(i18n("Can read all data in your home directory"));
                    }
                }
            }
            "host" => {
                permission_object = i18n("Host");
                icon_name = "drive-harddisk-symbolic".into();
                level = SkContextDetailLevel::Bad;
                is_folder = false;

                if subdir.is_none() {
                    if self.kind() == SkFilesystemPermissionKind::ReadOnly {
                        permission_title = Some(i18n("Full File System Read/Write Access"));
                        permission_description =
                            Some(i18n("Can read and write all data on the file system"));
                    } else {
                        permission_title = Some(i18n("Full File System Read-Only Access"));
                        permission_description = Some(i18n("Can read all data on the file system"));
                    }
                }
            }
            "host-os" => {
                permission_object = i18n("Host-os");
                icon_name = "drive-harddisk-symbolic".into();
                level = SkContextDetailLevel::Bad;
                is_folder = false;

                if subdir.is_none() {
                    permission_title =
                        Some(i18n("Full Access to System Libraries and Executables"));
                    permission_description = Some(i18n(
                        "Can access system libraries, executables and static data",
                    ));
                }
            }
            "host-etc" => {
                permission_object = i18n("Host-etc");
                icon_name = "drive-harddisk-symbolic".into();
                level = SkContextDetailLevel::Bad;
                is_folder = false;

                if subdir.is_none() {
                    permission_title = Some(i18n("Full Access to System Configuration"));
                    permission_description =
                        Some(i18n("Can access system configuration data from “/etc”"));
                }
            }
            "xdg-desktop" => {
                permission_object = i18n("Desktop");
                icon_name = "user-desktop-symbolic".into();
            }
            "xdg-documents" => {
                permission_object = i18n("Documents");
                icon_name = "emblem-documents-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            "xdg-download" => {
                permission_object = i18n("Downloads");
                icon_name = "folder-download-symbolic".into();
            }
            "xdg-music" => {
                permission_object = i18n("Music");
                icon_name = "folder-music-symbolic".into();
            }
            "xdg-pictures" => {
                permission_object = i18n("Pictures");
                icon_name = "folder-pictures-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            "xdg-public-share" => {
                permission_object = i18n("Public");
                icon_name = "folder-publicshare-symbolic".into();
            }
            "xdg-videos" => {
                permission_object = i18n("Videos");
                icon_name = "folder-videos-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            "xdg-templates" => {
                permission_object = i18n("Templates");
                icon_name = "folder-templates-symbolic".into();
            }
            "xdg-config" => {
                permission_object = i18n("Application Configuration");
                icon_name = "emblem-system-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            "xdg-cache" => {
                permission_object = i18n("Application Cache");
                icon_name = "folder-symbolic".into();
            }
            "xdg-data" => {
                permission_object = i18n("Application Data");
                icon_name = "folder-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            "xdg-run" => {
                permission_object = i18n("Runtime");
                icon_name = "system-run-symbolic".into();
                level = SkContextDetailLevel::Bad;
            }
            _ => {
                permission_object = permission;
                is_folder = false;

                warn!("Unknown permission object: {}", permission_object);

                // We don't know what this is about -> bad by default
                level = SkContextDetailLevel::Bad;
            }
        }

        if self.kind() != SkFilesystemPermissionKind::ReadOnly
            && path.contains("/flatpak/overrides")
        {
            permission_title = Some(i18n("Explicit Access to Flatpak System Folder"));
            permission_description = Some(i18n(
                "Can set arbitrary permissions, or change the permissions of other applications",
            ));
            level = SkContextDetailLevel::Bad;
        }

        let title_object_name = if is_folder {
            i18n_f("{} Folder", &[&permission_object])
        } else {
            permission_object.clone()
        };

        let title = if let Some(title) = permission_title {
            title
        } else if self.kind() == SkFilesystemPermissionKind::ReadOnly {
            i18n_f("{} Read-Only Access", &[&title_object_name])
        } else {
            i18n_f("{} Read/Write Access", &[&title_object_name])
        };

        let description = if let Some(description) = permission_description {
            description
        } else if self.kind() == SkFilesystemPermissionKind::ReadOnly {
            if let Some(subdir) = subdir {
                i18n_f(
                    "Can read “{}” in the “{}” directory",
                    &[&subdir, &permission_object],
                )
            } else if is_folder {
                i18n_f("Can read data in the “{}” directory", &[&permission_object])
            } else {
                i18n_f("Can read “{}”", &[&permission_object])
            }
        } else if let Some(subdir) = subdir {
            i18n_f(
                "Can read and write to “{}” in the “{}” directory",
                &[&subdir, &permission_object],
            )
        } else if is_folder {
            i18n_f(
                "Can read and write data to the “{}” directory",
                &[&permission_object],
            )
        } else {
            i18n_f("Can read and write to “{}”", &[&permission_object])
        };

        vec![SkContextDetail::new(
            kind,
            &icon_name,
            level,
            &title,
            &description,
        )]
    }
}
