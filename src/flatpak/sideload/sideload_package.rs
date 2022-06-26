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
use flatpak::Ref;
use gtk::glib::Bytes;
use gtk::prelude::*;

use crate::flatpak::context::SkContext;
use crate::worker::DryRunResult;

// TODO: This should be a gobject with properties
#[derive(Debug, Default, Clone)]
pub struct SideloadPackage {
    pub transaction_dry_run: DryRunResult,
}

impl SideloadPackage {
    pub fn ref_(&self) -> Ref {
        let ref_ = self.transaction_dry_run.ref_.clone();
        Ref::parse(&ref_).unwrap()
    }

    pub fn commit(&self) -> String {
        self.transaction_dry_run.commit.clone()
    }

    pub fn icon(&self) -> Option<gdk::Paintable> {
        let icon = self.transaction_dry_run.icon.clone();
        let bytes = Bytes::from_owned(icon);
        if let Ok(paintable) = gdk::Texture::from_bytes(&bytes) {
            Some(paintable.upcast())
        } else {
            None
        }
    }

    pub fn appstream(&self) -> Option<Component> {
        if let Some(appstream) = self.transaction_dry_run.appstream_component.as_ref() {
            serde_json::from_str(appstream).ok()
        } else {
            None
        }
    }

    pub fn download_size_context(&self) -> SkContext {
        SkContext::download_size(&self.transaction_dry_run)
    }

    pub fn installed_size_context(&self) -> SkContext {
        SkContext::installed_size(&self.transaction_dry_run)
    }

    pub fn has_update_source(&self) -> bool {
        self.transaction_dry_run.has_update_source
    }

    pub fn is_replacing_remote(&self) -> Option<String> {
        self.transaction_dry_run
            .is_replacing_remote
            .as_ref()
            .cloned()
    }

    pub fn is_already_installed(&self) -> bool {
        self.transaction_dry_run.is_already_installed
    }

    pub fn is_update(&self) -> bool {
        self.transaction_dry_run.is_update
    }
}
