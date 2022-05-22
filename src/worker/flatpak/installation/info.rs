// Souk - info.rs
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

use gtk::prelude::*;
use libflatpak::prelude::*;
use libflatpak::Installation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct InstallationInfo {
    pub uuid: String,
    pub id: String,
    pub display_name: String,
    pub is_user: bool,
    pub is_custom: bool,
    pub path: String,
}

impl InstallationInfo {
    pub fn new(installation: &Installation, is_custom: bool) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            id: installation.id().unwrap().to_string(),
            display_name: installation.display_name().unwrap().to_string(),
            is_user: installation.is_user(),
            is_custom,
            path: installation
                .path()
                .unwrap()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}
