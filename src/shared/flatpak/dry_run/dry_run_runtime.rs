// Souk - dry_run_runtime.rs
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
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use crate::shared::flatpak::info::PackageInfo;
use crate::shared::flatpak::FlatpakOperationType;

#[derive(Derivative, Default, Deserialize, Serialize, Type, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug)]
pub struct DryRunRuntime {
    pub package: PackageInfo,
    pub operation_type: FlatpakOperationType,

    pub download_size: u64,
    pub installed_size: u64,

    #[derivative(Debug = "ignore")]
    pub icon: Optional<Vec<u8>>,
    /// Json serialized appstream component
    #[derivative(Debug = "ignore")]
    pub appstream_component: Optional<String>,
    /// Flatpak metadata
    #[derivative(Debug = "ignore")]
    pub metadata: String,
    #[derivative(Debug = "ignore")]
    pub old_metadata: Optional<String>,
}
