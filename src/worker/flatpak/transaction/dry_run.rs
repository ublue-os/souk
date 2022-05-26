// Souk - dry_run_results.rs
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

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;
use zbus::DBusError;

#[derive(DBusError, Debug)]
pub enum DryRunError {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),

    RuntimeNotFound(String),
    Other(String),
}

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct DryRunResults {
    pub ref_: String,
    pub commit: String,
    pub icon: Vec<u8>,
    /// Json serialized appstream component
    pub appstream_component: String,
    /// Whether the package with the exact commit is already installed
    pub is_already_done: bool,
    /// The same ref is already installed, but the commit differs
    pub is_update: bool,
    /// Whether the package has an source for future app updates
    pub has_update_source: bool,
    /// Whether the package is already installed from a different remote, and
    /// the old app needs to get uninstalled first
    pub is_replacing_remote: String,

    pub download_size: u64,
    pub installed_size: u64,
    pub runtimes: Vec<DryRunRuntime>,
    pub remotes: Vec<DryRunRemote>,
}

impl DryRunResults {
    pub fn download_size(&self) -> u64 {
        let mut size = self.download_size;
        for runtime in &self.runtimes {
            size += runtime.download_size;
        }
        size
    }

    pub fn installed_size(&self) -> u64 {
        let mut size = self.installed_size;
        for runtime in &self.runtimes {
            size += runtime.installed_size;
        }
        size
    }
}

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct DryRunRuntime {
    pub ref_: String,
    pub type_: String,
    pub download_size: u64,
    pub installed_size: u64,
}

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct DryRunRemote {
    pub suggested_remote_name: String,
    pub url: String,
}
