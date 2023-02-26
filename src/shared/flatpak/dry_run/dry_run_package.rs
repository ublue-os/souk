// Souk - dry_run_package.rs
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
use flatpak::TransactionOperation;
use gtk::glib;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use crate::shared::flatpak::info::{PackageInfo, RemoteInfo};
use crate::shared::flatpak::FlatpakOperationKind;

#[derive(Derivative, Deserialize, Serialize, Type, Clone, PartialEq, Eq, Hash, glib::Boxed)]
#[boxed_type(name = "DryRunPackage")]
#[derivative(Debug)]
pub struct DryRunPackage {
    pub info: PackageInfo,
    pub operation_kind: FlatpakOperationKind,

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

impl DryRunPackage {
    pub fn from_flatpak_operation(
        operation: &TransactionOperation,
        remote_info: &RemoteInfo,
    ) -> Self {
        Self {
            info: PackageInfo::new(
                operation.get_ref().unwrap().to_string(),
                remote_info.clone(),
            ),
            operation_kind: operation.operation_type().into(),
            download_size: operation.download_size(),
            installed_size: operation.installed_size(),
            metadata: operation.metadata().unwrap().to_data().to_string(),
            old_metadata: operation.metadata().map(|m| m.to_data().to_string()).into(),
            ..Default::default()
        }
    }
}

impl Default for DryRunPackage {
    fn default() -> Self {
        Self {
            info: PackageInfo::default(),
            operation_kind: FlatpakOperationKind::default(),
            download_size: u64::default(),
            installed_size: u64::default(),
            icon: None.into(),
            appstream_component: None.into(),
            metadata: String::default(),
            old_metadata: None.into(),
        }
    }
}
