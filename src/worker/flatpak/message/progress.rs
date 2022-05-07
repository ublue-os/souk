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

    // The final ref is the last ref which gets handled in a set of operations in a transaction
    // and therefore usually the target/goal of a transaction.
    pub final_ref: String,
    pub current_ref: String,
    pub installation: String,

    pub current_stage: i32,
    pub total_stages: i32,
    pub operation_type: String,

    pub progress: i32,
    pub bytes_transferred: u64,
    pub start_time: u64,
}

impl Progress {
    pub fn new(
        transaction_uuid: String,
        transaction: &Transaction,
        operation: &TransactionOperation,
        operation_progress: &TransactionProgress,
    ) -> Self {
        let operations = transaction.operations();
        let installation = transaction.installation().unwrap();
        let op_index = operations.iter().position(|o| o == operation).unwrap();

        let final_ref = operations.last().unwrap().get_ref().unwrap().to_string();
        let current_ref = operation.get_ref().unwrap().to_string();
        let installation = installation.id().unwrap().to_string();
        let current_stage = (op_index + 1).try_into().unwrap();
        let total_stages = operations.len().try_into().unwrap();
        let operation_type = operation.operation_type().to_str().unwrap().to_string();
        let progress = operation_progress.progress();
        let bytes_transferred = operation_progress.bytes_transferred();
        let start_time = operation_progress.start_time();

        Self {
            transaction_uuid,
            final_ref,
            current_ref,
            installation,
            total_stages,
            current_stage,
            operation_type,
            progress,
            bytes_transferred,
            start_time,
        }
    }

    pub fn update(&self, operation_progress: &TransactionProgress) -> Self {
        let mut updated = self.clone();
        updated.progress = operation_progress.progress();
        updated.bytes_transferred = operation_progress.bytes_transferred();
        updated
    }
}
