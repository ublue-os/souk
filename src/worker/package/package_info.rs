// Souk - package_info.rs
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
use flatpak::Ref;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Type, Debug, Clone, Eq, PartialEq)]
pub struct PackageInfo {
    pub id: String,
    pub installation_id: String,
    pub remote_id: String,

    pub ref_: String,
    pub commit: String,
}

impl PackageInfo {
    pub fn new(ref_: &Ref, installation_id: &str, remote_id: &str) -> Self {
        let installation_id = installation_id.to_string();
        let remote_id = remote_id.to_string();
        let commit = ref_.commit().unwrap().to_string();
        let ref_ = ref_.format_ref().unwrap().to_string();

        let id = format!("{}{}{}{}", installation_id, remote_id, ref_, commit);
        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        let id = s.finish().to_string();

        Self {
            id,
            installation_id,
            remote_id,
            ref_,
            commit,
        }
    }
}
