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
use gtk::prelude::*;
use libflatpak::prelude::*;
use libflatpak::{Installation, Ref, Remote};

use super::{InstallationInfo, RemoteInfo};

// TODO: Error handling? Not found! :(

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

    pub fn launch_app(&self, installation_uuid: &str, ref_: &str, commit: &str) {
        debug!(
            "Launch app from installation \"{}\": {}",
            installation_uuid, ref_
        );

        let installation = self.installation_by_uuid(installation_uuid);
        let ref_ = Ref::parse(ref_).unwrap();

        let _ = installation.launch(
            &ref_.name().unwrap(),
            Some(&ref_.arch().unwrap()),
            Some(&ref_.branch().unwrap()),
            Some(commit),
            Cancellable::NONE,
        );
    }

    pub fn installations(&self) -> Vec<InstallationInfo> {
        let installations = self.installations.lock().unwrap();
        installations.values().cloned().collect()
    }

    // TODO: Expose this via the DBus interface / allow adding custom installations
    pub fn add_installation(&self, path: String, is_user: bool) -> InstallationInfo {
        debug!("Add new installation: {} ({:?})", path, is_user);

        let path = File::for_parse_name(&path);
        let installation = Installation::for_path(&path, is_user, Cancellable::NONE).unwrap();

        let info = InstallationInfo::new(&installation, false);
        self.installations
            .lock()
            .unwrap()
            .insert(info.uuid.clone(), info.clone());

        info
    }

    pub fn installation_by_uuid(&self, uuid: &str) -> Installation {
        let info = {
            let installations = self.installations.lock().unwrap();
            installations
                .get(uuid)
                .expect("Unknown installation uuid")
                .clone()
        };

        if !info.is_custom && info.is_user {
            Installation::new_user(Cancellable::NONE).unwrap()
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
            let installation = self.installation_by_uuid(&info.uuid);
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

    pub fn add_installation_remote(&self, installation_uuid: &str, repo_path: &str) {
        debug!(
            "Add remote for installation \"{}\": {}",
            installation_uuid, repo_path
        );

        let installation = self.installation_by_uuid(installation_uuid);
        let file = File::for_parse_name(repo_path);
        let bytes = file.load_bytes(Cancellable::NONE).unwrap().0;
        let remote = Remote::from_file("remote", &bytes).unwrap();

        // Determine remote name
        let name = if let Some(title) = remote.title() {
            title.to_lowercase()
        } else {
            file.basename()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_lowercase()
        };
        remote.set_name(Some(&name));

        installation
            .add_remote(&remote, true, Cancellable::NONE)
            .expect("Failed to add remote");
    }

    pub fn installation_remotes(&self, installation_uuid: &str) -> Vec<RemoteInfo> {
        let installation = self.installation_by_uuid(installation_uuid);
        let mut result = Vec::new();

        let remotes = installation.list_remotes(Cancellable::NONE);
        if let Err(err) = remotes {
            error!("Unable to list remotes: {}", err.message());
            return result;
        }

        for remote in remotes.unwrap() {
            let name = remote.name().unwrap();
            let repo = remote.url().unwrap();

            let mut remote_info = RemoteInfo::new(&name, &repo);
            remote_info.set_flatpak_remote(&remote);

            result.push(remote_info);
        }

        result
    }
}

impl Default for InstallationManager {
    fn default() -> Self {
        Self::new()
    }
}
