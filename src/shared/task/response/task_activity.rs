// Souk - task_activity.rs
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

use super::TaskStatus;
use crate::shared::flatpak::info::{PackageInfo, RemoteInfo};
use crate::shared::flatpak::FlatpakOperationKind;
use crate::shared::task::FlatpakTask;

#[derive(Default, Deserialize, Serialize, PartialEq, Debug, Clone, Eq, Hash)]
pub struct TaskActivity {
    pub index: u32,
    pub dependencies: Vec<TaskActivity>,

    pub status: TaskStatus,
    pub progress: i32,
    pub download_rate: u64,

    pub operation_kind: FlatpakOperationKind,
    pub package: Option<PackageInfo>,
}

impl TaskActivity {
    /// Creates a [TaskActivity] from a Flatpak [Transaction] with all
    /// operations included. For this the Flatpak [Transaction] needs to be in
    /// the `ready` or `ready-pre-auth` state.
    pub fn from_flatpak_transaction(task: &FlatpakTask, transaction: &Transaction) -> Self {
        let mut operation_activities = Vec::new();
        for operation in transaction.operations() {
            let mut activity =
                TaskActivity::flatpak_operation_activity(transaction, &operation, None, false);

            // uninstall_before_install -> Additional transaction in which the already
            // installed ref gets uninstalled first. All subsequent activities therefore
            // have an index shifted by 1.
            if task.uninstall_before_install {
                activity.index += 1;
            }

            operation_activities.push(activity);
        }

        if task.kind.targets_single_package() {
            let mut main_activity = operation_activities.pop().unwrap();
            main_activity.index = 0;
            main_activity.dependencies = operation_activities;
            main_activity
        } else {
            let mut activity = Self::flatpak_main_activity(None, operation_activities.len());
            activity.dependencies = operation_activities;
            activity
        }
    }

    /// Creates a [TaskActivity] for a Flatpak [TransactionOperation]
    pub fn from_flatpak_operation(
        task: &FlatpakTask,
        transaction: &Transaction,
        operation: &TransactionOperation,
        op_progress: Option<&TransactionProgress>,
        is_done: bool,
    ) -> Self {
        let ops = transaction.operations();
        let index: u32 = ops
            .iter()
            .position(|o| o == operation)
            .unwrap()
            .try_into()
            .unwrap();

        let mut activity =
            TaskActivity::flatpak_operation_activity(transaction, operation, op_progress, is_done);
        let is_last_operation = (index + 1) as usize == ops.len();

        if task.kind.targets_single_package() && is_last_operation {
            // The main activity always has index 0
            activity.index = 0;
            activity.progress = Self::calculate_progress(activity.progress, index, ops.len());
            activity
        } else {
            // uninstall_before_install -> Additional transaction in which the already
            // installed ref gets uninstalled first. All subsequent activities therefore
            // have an index shifted by 1.
            if task.uninstall_before_install {
                activity.index += 1;
            }

            let mut main_activity = Self::flatpak_main_activity(Some(&activity), ops.len());
            main_activity.dependencies = vec![activity];
            main_activity
        }
    }

    fn flatpak_operation_activity(
        transaction: &Transaction,
        operation: &TransactionOperation,
        op_progress: Option<&TransactionProgress>,
        is_done: bool,
    ) -> Self {
        let ops = transaction.operations();
        let index: u32 = ops
            .iter()
            .position(|o| o == operation)
            .unwrap()
            .try_into()
            .unwrap();

        let mut progress = 0;
        let mut download_rate = 0;
        let mut status = TaskStatus::Pending;

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
            status = TaskStatus::Done;
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
        let package = PackageInfo::new(ref_.to_string(), remote_info);
        let operation_kind = operation.operation_type().into();

        Self {
            index,
            dependencies: Vec::default(),
            status,
            progress,
            download_rate,
            operation_kind,
            package: Some(package),
        }
    }

    fn flatpak_main_activity(activity: Option<&Self>, operations: usize) -> Self {
        let mut main_activity = TaskActivity::default();

        if let Some(activity) = activity {
            main_activity.status = activity.status.clone();
            main_activity.download_rate = activity.download_rate;

            let progress = Self::calculate_progress(activity.progress, activity.index, operations);
            main_activity.progress = progress;
        }

        main_activity
    }

    fn calculate_progress(current_progress: i32, current_index: u32, operations: usize) -> i32 {
        ((((current_index * 100) + current_progress as u32) as f32 / (operations as f32 * 100.0))
            * 100.0) as i32
    }
}
