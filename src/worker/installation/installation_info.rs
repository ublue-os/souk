// Souk - installation_info.rs
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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use flatpak::prelude::*;
use flatpak::Installation;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

use crate::worker::installation::RemoteInfo;

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Default, Debug, Clone)]
pub struct InstallationInfo {
    pub id: String,
    pub name: String,
    pub title: String,
    pub is_user: bool,
    pub path: String,

    pub remotes: Vec<RemoteInfo>,
}

impl InstallationInfo {
    pub fn new(installation: &Installation) -> Self {
        let name = installation.id().unwrap().to_string();
        let title = installation.display_name().unwrap().to_string();
        let is_user = installation.is_user();
        let path = installation
            .path()
            .unwrap()
            .path()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let id = format!("{}{}{}", name, is_user, path);
        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        let id = s.finish().to_string();

        Self {
            id,
            name,
            title,
            is_user,
            path,
            remotes: Vec::default(),
        }
    }
}
