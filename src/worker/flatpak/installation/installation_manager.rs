// Souk - installation_manager.rs
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gio::{Cancellable, File};
use gtk::gio;
use libflatpak::prelude::*;
use libflatpak::Installation;

use super::InstallationInfo;

#[derive(Clone, Debug)]
pub struct InstallationManager {
    installations: Arc<Mutex<HashMap<String, InstallationInfo>>>,
}

impl InstallationManager {
    pub fn new() -> Self {
        let mut installations = HashMap::new();

        let installation = Installation::new_system(Cancellable::NONE).unwrap();
        let info = InstallationInfo::new(&installation, false);
        installations.insert(info.uuid.clone(), info);

        let installation = Installation::new_user(gio::Cancellable::NONE).unwrap();
        let info = InstallationInfo::new(&installation, false);
        installations.insert(info.uuid.clone(), info);

        Self {
            installations: Arc::new(Mutex::new(installations)),
        }
    }

    pub fn installations(&self) -> Vec<InstallationInfo> {
        let installations = self.installations.lock().unwrap();
        installations.values().cloned().collect()
    }

    // TODO: Expose this via the DBus interface / allow adding custom installations
    pub fn add_installation(&self, path: String, is_user: bool) -> InstallationInfo {
        let path = File::for_parse_name(&path);
        let installation = Installation::for_path(&path, is_user, Cancellable::NONE).unwrap();

        let info = InstallationInfo::new(&installation, false);
        self.installations
            .lock()
            .unwrap()
            .insert(info.uuid.clone(), info.clone());

        info
    }

    pub fn flatpak_installation_by_uuid(&self, uuid: &str) -> Installation {
        let info = {
            let installations = self.installations.lock().unwrap();
            installations
                .get(uuid)
                .expect("Unknown installation uuid")
                .clone()
        };

        if !info.is_custom && info.is_user {
            Installation::new_user(gio::Cancellable::NONE).unwrap()
        } else if !info.is_custom && !info.is_user {
            Installation::new_system(Cancellable::NONE).unwrap()
        } else {
            let path = File::for_parse_name(&info.path);
            Installation::for_path(&path, info.is_user, Cancellable::NONE).unwrap()
        }
    }

    /// Returns the id of the installation with the most installed refs
    pub fn preferred_installation(&self) -> InstallationInfo {
        let installations = { self.installations.lock().unwrap().clone() };

        let mut top_count = 0;
        let mut preferred = None;

        for info in installations.values() {
            let installation = self.flatpak_installation_by_uuid(&info.uuid);
            let count = installation
                .list_installed_refs(Cancellable::NONE)
                .unwrap()
                .len();

            if count > top_count {
                top_count = count;
                preferred = Some(info);
            }
        }

        preferred.unwrap().clone()
    }
}

impl Default for InstallationManager {
    fn default() -> Self {
        Self::new()
    }
}
