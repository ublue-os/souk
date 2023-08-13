// Souk - operation_activity.rs
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

use flatpak::prelude::*;
use flatpak::{Transaction, TransactionOperation, TransactionProgress};
use gtk::gio;
use serde::{Deserialize, Serialize};

use super::OperationStatus;
use crate::shared::appstream::AppstreamOperationKind;
use crate::shared::flatpak::info::{PackageInfo, RemoteInfo};
use crate::shared::flatpak::FlatpakOperationKind;

#[derive(Default, Deserialize, Serialize, PartialEq, Debug, Clone, Eq, Hash)]
pub struct OperationActivity {
    pub status: OperationStatus,
    pub progress: i32,
    pub download_rate: u64,

    pub flatpak_operation: FlatpakOperationKind,
    pub appstream_operation: AppstreamOperationKind,
    pub package: Option<PackageInfo>,
    pub remote: Option<RemoteInfo>,
}

impl OperationActivity {
    /// Creates a [OperationActivity] for a Flatpak [TransactionOperation]
    pub fn from_flatpak_operation(
        transaction: &Transaction,
        operation: &TransactionOperation,
        op_progress: Option<&TransactionProgress>,
        is_done: bool,
    ) -> Self {
        let mut progress = 0;
        let mut download_rate = 0;
        let mut status = OperationStatus::Pending;

        if let Some(op_progress) = op_progress {
            // Calculate download-rate in bytes per second
            let start_time = op_progress.start_time();
            let elapsed_time = (gtk::glib::monotonic_time() as u64 - start_time) as f64 / 1000000.0;
            let bytes_second = (op_progress.bytes_transferred() as f64 / elapsed_time) as u64;

            progress = op_progress.progress();
            download_rate = bytes_second;
            status = operation.operation_type().into();
        }

        if is_done {
            progress = 100;
            download_rate = 0;
            status = OperationStatus::Done;
        }

        // Retrieve remote info
        let installation = transaction.installation().unwrap();
        let remote_name = operation.remote().unwrap();
        let remote = installation
            .remote_by_name(&remote_name, gio::Cancellable::NONE)
            .unwrap();
        let remote_info = RemoteInfo::from_flatpak(&remote, &installation);

        // Retrieve package info
        let ref_ = operation.get_ref().unwrap();
        let package_info = PackageInfo::new(ref_.to_string(), remote_info.clone());
        let flatpak_operation = operation.operation_type().into();

        Self {
            status,
            progress,
            download_rate,
            flatpak_operation,
            package: Some(package_info),
            remote: Some(remote_info),
            ..Default::default()
        }
    }

    pub fn new_appstream(remote: Option<&RemoteInfo>, status: OperationStatus) -> Self {
        let appstream_operation = if remote.is_some() {
            AppstreamOperationKind::Sync
        } else {
            AppstreamOperationKind::Compile
        };

        let progress = if status == OperationStatus::Done {
            100
        } else {
            0
        };

        Self {
            status,
            progress,
            remote: remote.cloned(),
            appstream_operation,
            ..Default::default()
        }
    }

    /// Required for [SkOperationModel] - we can't use the task uuid here, since
    /// it wouldn't differ for different operations
    pub fn identifier(&self) -> String {
        if self.flatpak_operation != FlatpakOperationKind::None {
            format!("{:?}:{:?}", self.flatpak_operation, self.package)
        } else if self.appstream_operation != AppstreamOperationKind::None {
            format!("{:?}:{:?}", self.appstream_operation, self.remote)
        } else {
            error!("Unable to generate identifier for operation activity");
            String::new()
        }
    }
}
