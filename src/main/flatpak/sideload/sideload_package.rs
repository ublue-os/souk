// Souk - sideload_package.rs
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

use appstream::Component;
use gtk::glib::Bytes;
use gtk::prelude::*;

use crate::main::context::SkContext;
use crate::main::flatpak::permissions::SkAppPermissions;
use crate::worker::DryRunResult;

// TODO: This should be a gobject with properties
// TODO: Maybe we can split the metadata stuff into a own object which can be
// shared later with real packages
#[derive(Debug, Default, Clone)]
pub struct SideloadPackage {
    pub dry_run_result: DryRunResult,
}

impl SideloadPackage {
    // Appstream information
    pub fn icon(&self) -> Option<gdk::Paintable> {
        let icon = self.dry_run_result.icon.clone();
        let bytes = Bytes::from_owned(icon);
        if let Ok(paintable) = gdk::Texture::from_bytes(&bytes) {
            Some(paintable.upcast())
        } else {
            None
        }
    }

    // Appstream information
    pub fn appstream(&self) -> Option<Component> {
        if let Some(appstream) = self.dry_run_result.appstream_component.as_ref() {
            serde_json::from_str(appstream).ok()
        } else {
            None
        }
    }

    pub fn metadata(&self) -> String {
        self.dry_run_result.metadata.clone()
    }

    pub fn old_metadata(&self) -> Option<String> {
        self.dry_run_result.old_metadata.clone().into()
    }

    pub fn permissions(&self) -> SkAppPermissions {
        SkAppPermissions::from_metadata(&self.metadata())
    }

    pub fn old_permissions(&self) -> Option<SkAppPermissions> {
        self.old_metadata()
            .map(|m| SkAppPermissions::from_metadata(&m))
    }

    pub fn permissions_context(&self) -> SkContext {
        SkContext::permissions(&self.permissions())
    }

    pub fn download_size_context(&self) -> SkContext {
        SkContext::download_size(&self.dry_run_result)
    }

    pub fn installed_size_context(&self) -> SkContext {
        SkContext::installed_size(&self.dry_run_result)
    }

    // DryRun information
    pub fn has_update_source(&self) -> bool {
        self.dry_run_result.has_update_source
    }

    // DryRun information
    pub fn is_replacing_remote(&self) -> Option<String> {
        self.dry_run_result.is_replacing_remote.as_ref().cloned()
    }

    // DryRun information
    pub fn is_already_installed(&self) -> bool {
        self.dry_run_result.is_already_installed
    }

    // DryRun information
    pub fn is_update(&self) -> bool {
        self.dry_run_result.is_update
    }
}
