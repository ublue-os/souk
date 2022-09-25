// Souk - response.rs
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
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use crate::shared::task::{Response, ResponseType};
use crate::worker::DryRunResult;

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub struct FlatpakResponse {
    pub progress: i32,
    pub dry_run_result: Optional<DryRunResult>,
}

impl FlatpakResponse {
    pub fn new_transaction_progress(
        uuid: String,
        transaction: &Transaction,
        operation: &TransactionOperation,
        progress: Option<&TransactionProgress>,
    ) -> Response {
        let ops = transaction.operations();
        let ops_count: i32 = ops.len().try_into().unwrap();
        let current_op: i32 = ops
            .iter()
            .position(|o| o == operation)
            .unwrap()
            .try_into()
            .unwrap();

        let mut op_is_done = false;
        let op_percentage: i32 = if let Some(progress) = progress {
            progress.progress()
        } else {
            // If we don't have any progress -> assume the current operation is done
            op_is_done = true;
            100
        };

        let transaction_percentage =
            (((current_op * 100) + op_percentage) as f32 / (ops_count as f32 * 100.0)) * 100.0;

        let response = Self {
            progress: transaction_percentage as i32,
            dry_run_result: None.into(),
        };

        if ops_count == (current_op + 1) && op_is_done {
            Response::new_flatpak(uuid, ResponseType::Done, response)
        } else {
            Response::new_flatpak(uuid, ResponseType::Update, response)
        }
    }

    pub fn new_dry_run_result(uuid: String, dry_run_result: DryRunResult) -> Response {
        let response = Self {
            progress: 100,
            dry_run_result: Some(dry_run_result).into(),
        };
        Response::new_flatpak(uuid, ResponseType::Done, response)
    }
}

impl Default for FlatpakResponse {
    fn default() -> Self {
        Self {
            progress: -1,
            dry_run_result: None.into(),
        }
    }
}
