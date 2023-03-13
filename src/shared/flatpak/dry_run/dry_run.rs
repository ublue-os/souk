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

use gtk::glib;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use super::DryRunPackage;
use crate::shared::flatpak::info::RemoteInfo;

#[derive(Deserialize, Debug, Serialize, Type, Clone, PartialEq, Eq, Hash, glib::Boxed)]
#[boxed_type(name = "DryRun", nullable)]
pub struct DryRun {
    /// The Flatpak package for which the dry-run is performed (can be an
    /// application or runtime)
    pub package: DryRunPackage,

    /// Runtimes that would be affected by the Flatpak transaction (e.g.
    /// install, update or uninstall)
    pub runtimes: Vec<DryRunPackage>,
    /// Remotes that would be added by the Flatpak transaction
    pub remotes: Vec<RemoteInfo>,

    /// Whether the package has an source for future app updates (for example
    /// Flatpak bundles don't have necessary an update source)
    pub has_update_source: bool,
    /// Whether the package is already installed from a different remote, and
    /// the old app needs to get uninstalled first
    pub is_replacing_remote: Optional<RemoteInfo>,
}

impl Default for DryRun {
    fn default() -> Self {
        Self {
            package: DryRunPackage::default(),
            runtimes: Vec::default(),
            remotes: Vec::default(),
            has_update_source: true,
            is_replacing_remote: None.into(),
        }
    }
}
