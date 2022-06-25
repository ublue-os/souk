// Souk - transaction_dry_run.rs
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

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use crate::worker::installation::RemoteInfo;

#[derive(Derivative, Deserialize, Serialize, Type, Clone)]
#[derivative(Debug)]
pub struct TransactionDryRun {
    pub ref_: String,
    pub commit: String,
    #[derivative(Debug = "ignore")]
    pub icon: Vec<u8>,
    /// Json serialized appstream component
    #[derivative(Debug = "ignore")]
    pub appstream_component: Optional<String>,
    /// Whether the package with the exact commit is already installed
    pub is_already_installed: bool,
    /// The same ref is already installed, but the commit differs
    pub is_update: bool,
    /// Whether the package has an source for future app updates
    pub has_update_source: bool,
    /// Whether the package is already installed from a different remote, and
    /// the old app needs to get uninstalled first
    pub is_replacing_remote: Optional<String>,

    /// Size information of the actual package (size information about the
    /// runtimes are in `runtimes`)
    pub download_size: u64,
    pub installed_size: u64,

    /// Which runtimes are installed during the installation
    pub runtimes: Vec<TransactionDryRunRuntime>,
    /// Which remote may be added during installation
    pub remote: Optional<RemoteInfo>,
}

impl TransactionDryRun {
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

impl Default for TransactionDryRun {
    fn default() -> Self {
        Self {
            ref_: String::default(),
            commit: String::default(),
            icon: Vec::default(),
            appstream_component: None.into(),
            is_already_installed: false,
            is_update: false,
            has_update_source: true,
            is_replacing_remote: None.into(),
            download_size: 0,
            installed_size: 0,
            runtimes: Vec::default(),
            remote: None.into(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Type, Debug, Clone)]
pub struct TransactionDryRunRuntime {
    pub ref_: String,
    pub operation_type: String,
    pub download_size: u64,
    pub installed_size: u64,
}
