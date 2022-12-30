// Souk - task_step.rs
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

use flatpak::prelude::*;
use flatpak::{Transaction, TransactionOperation, TransactionProgress};
use gtk::gio;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

use crate::shared::info::{InstallationInfo, PackageInfo, RemoteInfo};

#[derive(Default, Deserialize, Serialize, Type, PartialEq, Debug, Clone)]
pub struct TaskStep {
    pub index: u32,
    pub progress: i32,
    pub download_rate: u64,
    pub activity: String,
    pub package_info: PackageInfo,
}

impl TaskStep {
    pub fn new_flatpak(
        transaction: &Transaction,
        operation: &TransactionOperation,
        op_progress: Option<&TransactionProgress>,
        is_done: bool,
    ) -> Self {
        let index: u32 = transaction
            .operations()
            .iter()
            .position(|o| o == operation)
            .unwrap()
            .try_into()
            .unwrap();

        let (progress, download_rate, activity) = if let Some(op_progress) = op_progress {
            // Calculate download-rate in bytes per second
            let start_time = op_progress.start_time();
            let elapsed_time = (gtk::glib::monotonic_time() as u64 - start_time) as f64 / 1000000.0;
            let bytes_second = (op_progress.bytes_transferred() as f64 / elapsed_time) as u64;

            (
                op_progress.progress(),
                bytes_second,
                operation.operation_type().to_str().unwrap().to_string(),
            )
        } else if is_done {
            (100, 0, "done".to_string())
        } else {
            (0, 0, "pending".to_string())
        };

        let ref_ = operation.get_ref().unwrap();

        let installation = transaction.installation().unwrap();
        let installation_info = InstallationInfo::from(&installation);

        let remote_name = operation.remote().unwrap();
        let remote = installation
            .remote_by_name(&remote_name, gio::Cancellable::NONE)
            .unwrap();
        let remote_info = RemoteInfo::from(&remote);

        let package_info = PackageInfo::new(ref_.to_string(), installation_info, remote_info);

        Self {
            index,
            progress,
            download_rate,
            activity,
            package_info,
        }
    }
}
