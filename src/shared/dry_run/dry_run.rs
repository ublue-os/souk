// Souk - dry_run
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

use derivative::Derivative;
use gtk::glib::{KeyFile, KeyFileFlags};
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use super::DryRunRuntime;
use crate::shared::info::{PackageInfo, RemoteInfo};

#[derive(Derivative, Deserialize, Serialize, Type, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug)]
pub struct DryRun {
    /// The affected package
    pub package: PackageInfo,

    /// Size information of the actual package (size information about the
    /// runtimes are in `runtimes`)
    pub download_size: u64,
    pub installed_size: u64,

    /// Which runtimes are installed during the installation
    pub runtimes: Vec<DryRunRuntime>,
    /// Which remotes are getting added during installation
    pub added_remotes: Vec<RemoteInfo>,

    #[derivative(Debug = "ignore")]
    pub icon: Vec<u8>,
    /// Json serialized appstream component
    #[derivative(Debug = "ignore")]
    pub appstream_component: Optional<String>,
    /// Flatpak metadata
    #[derivative(Debug = "ignore")]
    pub metadata: String,
    #[derivative(Debug = "ignore")]
    pub old_metadata: Optional<String>,

    /// Whether the package with the exact commit is already installed
    pub is_already_installed: bool,
    /// The same ref is already installed, but the commit differs
    pub is_update: bool,
    /// Whether the package has an source for future app updates (not always
    /// the case, for example sideloading a bundle)
    pub has_update_source: bool,
    /// Whether the package is already installed from a different remote, and
    /// the old app needs to get uninstalled first
    pub is_replacing_remote: Optional<RemoteInfo>,
}

impl DryRun {
    pub fn has_extra_data(&self) -> bool {
        let keyfile = KeyFile::new();
        let _ = keyfile.load_from_data(&self.metadata, KeyFileFlags::NONE);
        keyfile.has_group("Extra Data")
    }
}

impl Default for DryRun {
    fn default() -> Self {
        Self {
            package: PackageInfo::default(),
            runtimes: Vec::default(),
            download_size: 0,
            installed_size: 0,
            added_remotes: Vec::default(),
            icon: Vec::default(),
            appstream_component: None.into(),
            metadata: String::new(),
            old_metadata: None.into(),
            is_already_installed: false,
            is_update: false,
            has_update_source: true,
            is_replacing_remote: None.into(),
        }
    }
}
