// Souk - installation_info.rs
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

use flatpak::prelude::*;
use flatpak::Installation;
use gtk::prelude::*;
use gtk::{gio, glib};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Eq, PartialEq, Default, Debug, Clone, Hash, glib::Boxed)]
#[boxed_type(name = "InstallationInfo", nullable)]
pub struct InstallationInfo {
    /// Flatpak installation id/name. For example `default` or `user`.
    pub name: String,
    /// Whether a Flatpak installation is for the current user only, or
    /// system-wide. System-wide installations require elevated privileges to
    /// make changes.
    pub is_user: bool,
    /// The path of the Flatpak OSTree repository.
    pub path: String,
}

impl From<&Installation> for InstallationInfo {
    fn from(installation: &Installation) -> Self {
        let name = installation.id().unwrap().to_string();
        let is_user = installation.is_user();
        let path = installation
            .path()
            .unwrap()
            .path()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Self {
            name,
            is_user,
            path,
        }
    }
}

impl From<&InstallationInfo> for Installation {
    fn from(info: &InstallationInfo) -> Self {
        if info.name == "user" && info.is_user {
            let mut user_path = glib::home_dir();
            user_path.push(".local");
            user_path.push("share");
            user_path.push("flatpak");
            let file = gio::File::for_path(user_path);

            Installation::for_path(&file, true, gio::Cancellable::NONE)
                .expect("Unable to create Flatpak system installation for path")
        } else if info.name == "default" && !info.is_user {
            Installation::new_system(gio::Cancellable::NONE)
                .expect("Unable to create Flatpak system installation")
        } else if !info.is_user {
            Installation::new_system_with_id(Some(&info.name), gio::Cancellable::NONE)
                .expect("Unable to create Flatpak system installation with id")
        } else {
            let path = gio::File::for_parse_name(&info.path);
            Installation::for_path(&path, info.is_user, gio::Cancellable::NONE)
                .expect("Unable to create Flatpak user installation for path")
        }
    }
}
