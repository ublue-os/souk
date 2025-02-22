// Souk - operation_kind.rs
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

use flatpak::TransactionOperationType;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum FlatpakOperationKind {
    Install,
    InstallBundle,
    Uninstall,
    Update,
    #[default]
    None,
}

impl From<TransactionOperationType> for FlatpakOperationKind {
    fn from(op: TransactionOperationType) -> Self {
        match op {
            TransactionOperationType::Install => Self::Install,
            TransactionOperationType::Update => Self::Update,
            TransactionOperationType::InstallBundle => Self::InstallBundle,
            TransactionOperationType::Uninstall => Self::Uninstall,
            _ => {
                warn!(
                    "Unknown Flatpak transaction operation type {}",
                    op.to_str().unwrap()
                );
                Self::None
            }
        }
    }
}
