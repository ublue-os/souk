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

use async_std::process::Command;
use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use gio::{Cancellable, File};
use gtk::prelude::*;
use gtk::{gio, glib};

use super::{InstallationInfo, RemoteInfo};
use crate::worker::{PackageInfo, WorkerError};

#[derive(Clone, Debug)]
pub struct InstallationManager {
    installations: Arc<Mutex<HashMap<String, InstallationInfo>>>,
}

impl InstallationManager {
    pub fn new() -> Self {
        let manager = Self {
            installations: Arc::default(),
        };

        manager.update_installations();
        manager
    }

    pub fn launch_app(
        &self,
        installation_id: &str,
        ref_: &str,
        // TODO: Check if we need the exact commit at all.
        _commit: &str,
    ) -> Result<(), WorkerError> {
        debug!(
            "Launch app from installation \"{}\": {}",
            installation_id, ref_
        );

        let installation = self.flatpak_installation_by_id(installation_id)?;
        let id = installation.id().unwrap().to_string();
        let installation = match id.as_str() {
            "user" => "--user".to_string(),
            "system" => "--system".to_string(),
            _ => format!("--installation={}", id),
        };

        Command::new("flatpak-spawn")
            .arg("--host")
            .arg("flatpak")
            .arg("run")
            .arg(installation)
            .arg(ref_)
            .spawn()
            .unwrap();

        Ok(())
    }

    /// Returns all installations
    pub fn installations(&self) -> Vec<InstallationInfo> {
        self.update_installations();
        let installations = self.installations.lock().unwrap();
        installations.values().cloned().collect()
    }

    /// Returns a libflatpak `Installation` by the installation id.
    pub fn installation_by_id(&self, installation_id: &str) -> Option<InstallationInfo> {
        let installations = self.installations.lock().unwrap();
        installations.get(installation_id).cloned()
    }

    /// Returns a libflatpak `Installation` by the installation id.
    pub(crate) fn flatpak_installation_by_id(&self, id: &str) -> Result<Installation, WorkerError> {
        let info = {
            let installations = self.installations.lock().unwrap();
            installations
                .get(id)
                .expect("Unknown installation id")
                .clone()
        };

        let installation = if info.id == "user" && info.is_user {
            let mut user_path = glib::home_dir();
            user_path.push(".local");
            user_path.push("share");
            user_path.push("flatpak");
            let file = gio::File::for_path(user_path);

            Installation::for_path(&file, true, Cancellable::NONE)?
        } else if info.id == "default" && !info.is_user {
            Installation::new_system(Cancellable::NONE)?
        } else {
            let path = File::for_parse_name(&info.path);
            Installation::for_path(&path, info.is_user, Cancellable::NONE)?
        };

        Ok(installation)
    }

    /// Returns the id of the installation with the most installed refs
    pub fn preferred_installation(&self) -> Result<InstallationInfo, WorkerError> {
        let installations = { self.installations.lock().unwrap().clone() };

        let mut top_count = 0;
        let mut preferred = None;

        for info in installations.values() {
            let installation = self.flatpak_installation_by_id(&info.id)?;
            let count = installation
                .list_installed_refs(Cancellable::NONE)
                .unwrap()
                .len();

            if count >= top_count {
                top_count = count;
                preferred = Some(info);
            }
        }

        Ok(preferred.unwrap().clone())
    }

    pub fn add_remote(&self, installation_id: &str, repo_path: &str) -> Result<(), WorkerError> {
        debug!(
            "Add remote for installation \"{}\": {}",
            installation_id, repo_path
        );

        let installation = self.flatpak_installation_by_id(installation_id)?;
        let file = File::for_parse_name(repo_path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;
        let remote = Remote::from_file("remote", &bytes)?;
        remote.set_gpg_verify(true);

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

        installation.add_remote(&remote, true, Cancellable::NONE)?;
        self.update_installations();

        Ok(())
    }

    fn update_installations(&self) {
        debug!("Updating Flatpak installations...");

        let mut installations = self.installations.lock().unwrap();
        installations.clear();

        let mut flatpak_installations = Vec::new();

        // System Installation
        let installation = Installation::new_system(Cancellable::NONE).unwrap();
        flatpak_installations.push(installation);

        // User Installation
        let mut user_path = glib::home_dir();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        let file = gio::File::for_path(user_path);
        let installation = Installation::for_path(&file, true, Cancellable::NONE).unwrap();
        flatpak_installations.push(installation);

        // Other Installations
        let mut system_installations =
            flatpak::functions::system_installations(Cancellable::NONE).unwrap();
        flatpak_installations.append(&mut system_installations);

        for flatpak_installation in flatpak_installations {
            let mut installation_info = InstallationInfo::new(&flatpak_installation);

            let mut remote_infos = Vec::new();
            let mut package_infos = Vec::new();

            let remotes = flatpak_installation
                .list_remotes(Cancellable::NONE)
                .unwrap();

            for remote in &remotes {
                let remote_info = RemoteInfo::new(remote, &installation_info.id);
                remote_infos.push(remote_info);
            }

            let installed_refs = flatpak_installation
                .list_installed_refs(Cancellable::NONE)
                .unwrap();
            for ref_ in installed_refs {
                let origin = ref_.origin().unwrap().to_string();
                let remote = flatpak_installation
                    .remote_by_name(&origin, Cancellable::NONE)
                    .unwrap();
                let remote_info = RemoteInfo::new(&remote, &installation_info.id);

                let package_info =
                    PackageInfo::new(&ref_.upcast(), &installation_info.id, &remote_info.id);
                package_infos.push(package_info);
            }

            installation_info.remotes = remote_infos;
            installation_info.packages = package_infos;
            installations.insert(installation_info.id.clone(), installation_info);
        }
    }
}

impl Default for InstallationManager {
    fn default() -> Self {
        Self::new()
    }
}
