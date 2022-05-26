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

use appstream::Component;
use async_trait::async_trait;
use gtk::gio::File;
use gtk::glib::Bytes;
use gtk::prelude::*;
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

    fn commit(&self) -> String {
        self.dry_run_results.commit.clone()
    }

    fn icon(&self) -> Option<gdk::Paintable> {
        let bytes = Bytes::from_owned(self.dry_run_results.icon.clone());
        if let Ok(paintable) = gdk::Texture::from_bytes(&bytes) {
            Some(paintable.upcast())
        } else {
            None
        }
    }

    fn appstream_component(&self) -> Option<Component> {
        serde_json::from_str(&self.dry_run_results.appstream_component).ok()
    }

    fn installation_uuid(&self) -> String {
        self.installation_uuid.clone()
    }

    fn has_update_source(&self) -> bool {
        self.dry_run_results.has_update_source
    }

    fn is_replacing_remote(&self) -> String {
        self.dry_run_results.is_replacing_remote.clone()
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
        let no_update = !self.is_replacing_remote().is_empty();

        let transaction = worker
            .install_flatpak_bundle(&self.ref_(), &self.file, &self.installation_uuid, no_update)
            .await?;
        Ok(transaction)
    }
}
