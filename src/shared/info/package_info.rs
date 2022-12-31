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

use std::hash::Hash;

use flatpak::prelude::*;
use flatpak::{Installation, InstalledRef, Remote};
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

use crate::shared::info::RemoteInfo;

#[derive(Default, Deserialize, Serialize, Type, Debug, Clone, Eq, PartialEq, Hash)]
pub struct PackageInfo {
    pub ref_: String,
    pub remote: RemoteInfo,
}

impl PackageInfo {
    pub fn new(ref_: String, remote: RemoteInfo) -> Self {
        Self { ref_, remote }
    }

    pub fn from_flatpak(
        installed_ref: &InstalledRef,
        remote: &Remote,
        installation: &Installation,
    ) -> Self {
        let ref_ = installed_ref.format_ref().unwrap().to_string();
        let remote = RemoteInfo::from_flatpak(remote, Some(installation));

        Self { ref_, remote }
    }
}
