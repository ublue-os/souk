// Souk - bundle.rs
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

use async_trait::async_trait;
use gtk::gio::File;
use libflatpak::Ref;

use crate::error::Error;
use crate::flatpak::sideload::{Sideloadable, SkSideloadType};
use crate::flatpak::{SkTransaction, SkWorker};
use crate::worker::DryRunResults;

#[derive(Debug)]
pub struct BundleSideloadable {
    file: File,
    dry_run_results: DryRunResults,
    installation_uuid: String,
}

impl BundleSideloadable {
    pub fn new(file: &File, dry_run_results: DryRunResults, installation_uuid: &str) -> Self {
        Self {
            file: file.clone(),
            dry_run_results,
            installation_uuid: installation_uuid.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl Sideloadable for BundleSideloadable {
    fn type_(&self) -> SkSideloadType {
        SkSideloadType::Bundle
    }

    fn contains_package(&self) -> bool {
        true
    }

    fn contains_repository(&self) -> bool {
        false // TODO: Bundles may include a repo
    }

    fn ref_(&self) -> Ref {
        Ref::parse(&self.dry_run_results.ref_).unwrap()
    }

    fn is_already_done(&self) -> bool {
        self.dry_run_results.is_already_done
    }

    fn is_update(&self) -> bool {
        self.dry_run_results.is_update
    }

    fn download_size(&self) -> u64 {
        self.dry_run_results.download_size
    }

    fn installed_size(&self) -> u64 {
        self.dry_run_results.installed_size
    }

    async fn sideload(&self, worker: &SkWorker) -> Result<SkTransaction, Error> {
        let transaction = worker
            .install_flatpak_bundle(&self.ref_(), &self.file, &self.installation_uuid)
            .await?;
        Ok(transaction)
    }
}
