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

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct DryRunResults {
    pub download_size: u64,
    pub installed_size: u64,
    pub runtimes: Vec<DryRunRuntime>,
    pub remotes: Vec<DryRunRemote>,

    pub is_error: bool,
    pub error_message: String,
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
