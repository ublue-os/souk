// Souk - progress.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

use libflatpak::prelude::*;
use libflatpak::{Transaction, TransactionOperation, TransactionProgress};
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Type, Default, Debug, Clone)]
pub struct Progress {
    pub transaction_uuid: String,

    pub ref_: String,
    pub type_: String,

    pub progress: i32,
    pub is_done: bool,
    pub bytes_transferred: u64,
    pub start_time: u64,

    pub current_operation: i32,
    pub operations_count: i32,
}

impl Progress {
    pub fn new(
        transaction_uuid: String,
        transaction: &Transaction,
        operation: &TransactionOperation,
        operation_progress: Option<&TransactionProgress>,
    ) -> Self {
        let operations = transaction.operations();
        let op_index = operations.iter().position(|o| o == operation).unwrap();

        let ref_ = operation.get_ref().unwrap().to_string();
        let type_ = operation.operation_type().to_str().unwrap().to_string();

        let current_operation = (op_index + 1).try_into().unwrap();
        let operations_count = operations.len().try_into().unwrap();

        let progress = Self {
            transaction_uuid,
            ref_,
            type_,
            current_operation,
            operations_count,
            ..Default::default()
        };

        if let Some(operation_progress) = operation_progress {
            progress.update(operation_progress)
        } else {
            progress
        }
    }

    pub fn update(&self, operation_progress: &TransactionProgress) -> Self {
        let mut updated = self.clone();
        updated.progress = operation_progress.progress();
        updated.bytes_transferred = operation_progress.bytes_transferred();
        updated.start_time = operation_progress.start_time();
        updated
    }

    pub fn done(&self) -> Self {
        let mut done = self.clone();
        done.is_done = true;
        done
    }
}
