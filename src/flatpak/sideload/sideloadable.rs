// Souk - sideloadable.rs
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

use core::fmt::Debug;

use async_trait::async_trait;
use libflatpak::prelude::*;
use libflatpak::Ref;

use crate::error::Error;
use crate::flatpak::sideload::SkSideloadType;
use crate::flatpak::{SkTransaction, SkWorker};

#[async_trait(?Send)]
pub trait Sideloadable {
    fn type_(&self) -> SkSideloadType;

    fn contains_package(&self) -> bool;

    fn contains_repository(&self) -> bool;

    fn ref_(&self) -> Ref;

    fn already_done(&self) -> bool;

    fn download_size(&self) -> u64;

    fn installed_size(&self) -> u64;

    async fn sideload(&self, worker: &SkWorker) -> Result<SkTransaction, Error>;
}

impl Debug for dyn Sideloadable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Sideloadable: {}", self.ref_().format_ref().unwrap())
    }
}
