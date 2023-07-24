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
        let kind = SkContextDetailKind::Icon;
        let icon_name = "folder-documents-symbolic".to_string();
        let level = SkContextDetailLevel::Good;

        let title = i18n("No Filessystem Access");
        let description = i18n("Cannot access the filesystem at all");

        SkContextDetail::new(kind, &icon_name, level, &title, &description)
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
        let path = self.path();

        vec![Details::new(self.kind().can_write(), &path).into()]
    }
}

enum Details<'a> {
    Home {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Host(bool),
    HostOs(bool),
    HostEtc(bool),
    Desktop {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Documents {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Download {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Music {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Pictures {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Public {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Videos {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Templates {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Config {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Cache {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Data {
        can_write: bool,
        subdir: Option<&'a str>,
    },
    Runtime {
        can_write: bool,
        name: &'a str,
    },
    Path {
        can_write: bool,
        path: &'a str,
    },
}

impl<'a> Details<'a> {
    fn new(can_write: bool, path: &'a str) -> Self {
        let (permission, subdir) = match path.split_once('/') {
            Some(("", path)) => (path, None),
            Some((path, subdir)) => (path, Some(subdir)),
            None => (path, None),
        };

        match permission {
            "home" | "~" => Self::Home { can_write, subdir },
            "host" => Self::Host(can_write),
            "host-os" => Self::HostOs(can_write),
            "host-etc" => Self::HostEtc(can_write),
            "xdg-desktop" => Self::Desktop { can_write, subdir },
            "xdg-documents" => Self::Documents { can_write, subdir },
            "xdg-download" => Self::Download { can_write, subdir },
            "xdg-music" => Self::Music { can_write, subdir },
            "xdg-pictures" => Self::Pictures { can_write, subdir },
            "xdg-public-share" => Self::Public { can_write, subdir },
            "xdg-videos" => Self::Videos { can_write, subdir },
            "xdg-templates" => Self::Templates { can_write, subdir },
            "xdg-config" => Self::Config { can_write, subdir },
            "xdg-cache" => Self::Cache { can_write, subdir },
            "xdg-data" => Self::Data { can_write, subdir },
            "xdg-run" => Self::Runtime {
                can_write,
                name: subdir.unwrap_or("*"),
            },
            _ => Self::Path { can_write, path },
        }
    }

    const fn icon_name(&self) -> &'static str {
        match self {
            Self::Home { .. } => "user-home-symbolic",
            Self::Host(_) | Self::HostOs(_) | Self::HostEtc(_) => "drive-harddisk-symbolic",
            Self::Desktop { .. } => "user-desktop-symbolic",
            Self::Documents { .. } => "user-documents-symbolic",
            Self::Download { .. } => "folder-download-symbolic",
            Self::Music { .. } => "folder-music-symbolic",
            Self::Pictures { .. } => "folder-pictures-symbolic",
            Self::Public { .. } => "folder-publicshare-symbolic",
            Self::Videos { .. } => "folder-videos-symbolic",
            Self::Templates { .. } => "folder-templates-symbolic",
            Self::Config { .. } => "emblem-system-symbolic",
            Self::Cache { .. } => "folder-symbolic",
            Self::Data { .. } => "folder-symbolic",
            Self::Runtime { .. } => "system-run-symbolic",
            Self::Path { .. } => "folder-symbolic",
        }
    }

    fn level(&self) -> SkContextDetailLevel {
        match self {
            Self::Runtime {
                name: "app/com.discordapp.Discord",
                ..
            } => SkContextDetailLevel::Moderate,
            Self::Home { .. }
            | Self::Host(_)
            | Self::HostOs(_)
            | Self::HostEtc(_)
            | Self::Documents { .. }
            | Self::Pictures { .. }
            | Self::Videos { .. }
            | Self::Config { .. }
            | Self::Data { .. }
            | Self::Runtime { .. } => SkContextDetailLevel::Bad,
            Self::Desktop {
                can_write: false, ..
            }
            | Self::Download {
                can_write: false, ..
            }
            | Self::Music {
                can_write: false, ..
            }
            | Self::Public {
                can_write: false, ..
            }
            | Self::Templates {
                can_write: false, ..
            }
            | Self::Cache {
                can_write: false, ..
            } => SkContextDetailLevel::Moderate,
            Self::Desktop { .. }
            | Self::Download { .. }
            | Self::Music { .. }
            | Self::Public { .. }
            | Self::Templates { .. }
            | Self::Cache { .. } => SkContextDetailLevel::Warning,
            // We don't know what this is about -> bad by default
            Self::Path { .. } => SkContextDetailLevel::Bad,
        }
    }

    fn describe(&self) -> (String, String) {
        match self {
            Self::Home {
                can_write: false,
                subdir: None,
            } => (
                i18n("Home Folder Read-Only Access"),
                i18n("Can read all data in your home directory"),
            ),
            Self::Home { subdir: None, .. } => (
                i18n("Home Folder Read/Write Access"),
                i18n("Can read and write all data in your home directory"),
            ),
            Self::Home {
                can_write: false,
                subdir: Some(dir),
            } => (
                i18n("Home Folder Read-Only Access"),
                i18n_f("Can read data under “{}” in your home directory", &[dir]),
            ),
            Self::Home {
                subdir: Some(dir), ..
            } => (
                i18n("Home Folder Read/Write Access"),
                i18n_f(
                    "Can read and write data under “{}” in your home directory",
                    &[dir],
                ),
            ),
            Self::Host(false) => (
                i18n("Full File System Read-Only Access"),
                i18n("Can read all data in the file system"),
            ),
            Self::Host(true) => (
                i18n("Full File System Read/Write Access"),
                i18n("Can read and write all data in the file system"),
            ),
            Self::HostOs(false) => (
                i18n("Full Access to System Libraries and Executables"),
                i18n("Can access system libraries, executables and static data"),
            ),
            Self::HostOs(true) => (
                i18n("Full Access to System Libraries and Executables"),
                i18n("Can access and modify system libraries, executables and static data"),
            ),
            Self::HostEtc(false) => (
                i18n("Full Access to System Configuration"),
                i18n("Can access system configuration data from “/etc”"),
            ),
            Self::HostEtc(true) => (
                i18n("Full Access to System Configuration"),
                i18n("Can access and modify system configuration data under “/etc”"),
            ),
            Self::Desktop { can_write: true, subdir: None } => (
                i18n("Desktop Folder Read/Write Access"),
                i18n("Can read and write data in “Desktop”")
            ),
            Self::Desktop { can_write: false, subdir: None } => (
                i18n("Desktop Folder Read-Only Access"),
                i18n("Can read data in “Desktop”")
            ),
            Self::Desktop { can_write: true, subdir: Some(dir) } => (
                i18n_f("Desktop Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data in “{}” on the Desktop", &[dir])
            ),
            Self::Desktop { can_write: false, subdir: Some(dir) } => (
                i18n_f("Desktop Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data in “{}” on the Desktop", &[dir])
            ),
            Self::Documents { can_write: true, subdir: None } => (
                i18n("Documents Folder Read/Write Access"),
                i18n("Can read and write data in Documents")
            ),
            Self::Documents { can_write: false, subdir: None } => (
                i18n("Documents Folder Read-Only Access"),
                i18n("Can read data in Documents")
            ),
            Self::Documents { can_write: true, subdir: Some(dir) } => (
                i18n_f("Documents Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Documents", &[dir])
            ),
            Self::Documents { can_write: false, subdir: Some(dir) } => (
                i18n_f("Documents Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Documents", &[dir])
            ),
            Self::Download { can_write: true, subdir: None } => (
                i18n("Download Folder Read/Write Access"),
                i18n("Can read and write data in Downloads")
            ),
            Self::Download { can_write: false, subdir: None } => (
                i18n("Download Folder Read-Only Access"),
                i18n("Can read data in Downloads")
            ),
            Self::Download { can_write: true, subdir: Some(dir) } => (
                i18n_f("Downloads Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Downloads", &[dir])
            ),
            Self::Download { can_write: false, subdir: Some(dir) } => (
                i18n_f("Downloads Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Downloads", &[dir])
            ),
            Self::Music { can_write: true, subdir: None } => (
                i18n("Music Folder Read/Write Access"),
                i18n("Can read and write data in Music")
            ),
            Self::Music { can_write: false, subdir: None } => (
                i18n("Music Folder Read-Only Access"),
                i18n("Can read data in Music")
            ),
            Self::Music { can_write: true, subdir: Some(dir) } => (
                i18n_f("Music Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Music", &[dir])
            ),
            Self::Music { can_write: false, subdir: Some(dir) } => (
                i18n_f("Music Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Music", &[dir])
            ),
            Self::Pictures { can_write: true, subdir: None } => (
                i18n("Pictures Folder Read/Write Access"),
                i18n("Can read and write data in Pictures")
            ),
            Self::Pictures { can_write: false, subdir: None } => (
                i18n("Pictures Folder Read-Only Access"),
                i18n("Can read data in Pictures")
            ),
            Self::Pictures { can_write: true, subdir: Some(dir) } => (
                i18n_f("Pictures Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Pictures", &[dir])
            ),
            Self::Pictures { can_write: false, subdir: Some(dir) } => (
                i18n_f("Pictures Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Pictures", &[dir])
            ),
            Self::Public { can_write: true, subdir: None } => (
                i18n("Public Folder Read/Write Access"),
                i18n("Can read and write data in Public")
            ),
            Self::Public { can_write: false, subdir: None } => (
                i18n("Public Folder Read-Only Access"),
                i18n("Can read data in Public")
            ),
            Self::Public { can_write: true, subdir: Some(dir) } => (
                i18n_f("Public Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Public", &[dir])
            ),
            Self::Public { can_write: false, subdir: Some(dir) } => (
                i18n_f("Public Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Public", &[dir])
            ),
            Self::Videos { can_write: true, subdir: None } => (
                i18n("Videos Folder Read/Write Access"),
                i18n("Can read and write data in Videos")
            ),
            Self::Videos { can_write: false, subdir: None } => (
                i18n("Videos Folder Read-Only Access"),
                i18n("Can read data in Videos")
            ),
            Self::Videos { can_write: true, subdir: Some(dir) } => (
                i18n_f("Videos Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Videos", &[dir])
            ),
            Self::Videos { can_write: false, subdir: Some(dir) } => (
                i18n_f("Videos Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Videos", &[dir])
            ),
            Self::Templates { can_write: true, subdir: None } => (
                i18n("Templates Folder Read/Write Access"),
                i18n("Can read and write data in Templates")
            ),
            Self::Templates { can_write: false, subdir: None } => (
                i18n("Templates Folder Read-Only Access"),
                i18n("Can read data in Templates")
            ),
            Self::Templates { can_write: true, subdir: Some(dir) } => (
                i18n_f("Templates Folder “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data under “{}” in Templates", &[dir])
            ),
            Self::Templates { can_write: false, subdir: Some(dir) } => (
                i18n_f("Templates Folder “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under “{}” in Templates", &[dir])
            ),
            Self::Config { can_write: true, subdir: None } => (
                i18n("Application Configuration Read/Write Access"),
                i18n("Can read and write all Application Configuration")
            ),
            Self::Config { can_write: false, subdir: None } => (
                i18n("Application Configuration Folder Read-Only Access"),
                i18n("Can read all Application Configuration")
            ),
            Self::Config { can_write: true, subdir: Some(dir) } => (
                i18n_f("Application Configuration “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write “{}” Application Configuration", &[dir])
            ),
            Self::Config { can_write: false, subdir: Some(dir) } => (
                i18n_f("Application Configuration “{}” Read-Only Access", &[dir]),
                i18n_f("Can read “{}” Application Configuration", &[dir])
            ),
            Self::Cache { can_write: true, subdir: None } => (
                i18n("Application Cache Read/Write Access"),
                i18n("Can read and write data in the Application Cache")
            ),
            Self::Cache { can_write: false, subdir: None } => (
                i18n("Application Cache Read-Only Access"),
                i18n("Can read data in the Application Cache")
            ),
            Self::Cache { can_write: true, subdir: Some(dir) } => (
                i18n_f("Application Cache “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write data in the “{}” Application Cache", &[dir])
            ),
            Self::Cache { can_write: false, subdir: Some(dir) } => (
                i18n_f("Application Cache “{}” Read-Only Access", &[dir]),
                i18n_f("Can read data under the “{}” Application Cache", &[dir])
            ),
            Self::Data { can_write: true, subdir: None } => (
                i18n("Application Data Read/Write Access"),
                i18n("Can read and write all Application Data")
            ),
            Self::Data { can_write: false, subdir: None } => (
                i18n("Application Data Read-Only Access"),
                i18n("Can read all Application Data")
            ),
            Self::Data { can_write: true, subdir: Some(dir) } => (
                i18n_f("Application Data “{}” Read/Write Access", &[dir]),
                i18n_f("Can read and write in “{}” Application Data", &[dir])
            ),
            Self::Data { can_write: false, subdir: Some(dir) } => (
                i18n_f("Application Data “{}” Read-Only Access", &[dir]),
                i18n_f("Can read “{}” Application Data", &[dir])
            ),
            Self::Runtime {  name: "app/com.discordapp.Discord", .. } => (
                i18n("Discord IPC Access"),
                i18n("Can interact with Discord")
            ),
            Self::Runtime { can_write: true, name } => (
                i18n_f("“{}” Runtime Folder Read/Write Access", &[name]),
                i18n_f("Can read and write runtime under “{}”", &[name])
            ),
            Self::Runtime { can_write: false, name } => (
                i18n_f("“{}” Runtime Folder Read-Only Access", &[name]),
                i18n_f("Can read runtime under “{}”", &[name])
            ),
            Self::Path { can_write: true, path } if path.contains("/flatpak/overrides")  => (
                i18n("Explicit Access to Flatpak System Folder"),
                i18n("Can set arbitrary permissions, or change the permissions of other applications")
            ),
            Self::Path { can_write: true, path } => (
                i18n_f("“{}” Read/Write Access", &[path]),
                i18n_f("Can read and write “{}”", &[path])
            ),
            Self::Path { can_write: false, path } => (
                i18n_f("“{}” Read-Only Access", &[path]),
                i18n_f("Can read “{}”", &[path])
            ),
        }
    }
}

impl<'a> From<Details<'a>> for SkContextDetail {
    fn from(value: Details) -> Self {
        let (title, description) = value.describe();
        Self::new(
            SkContextDetailKind::Icon,
            value.icon_name(),
            value.level(),
            &title,
            &description,
        )
    }
}
