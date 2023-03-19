// Souk - task.rs
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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::flatpak::info::{InstallationInfo, PackageInfo, RemoteInfo};
use crate::shared::task::{Task, TaskKind};

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
// TODO: This could be simplified by using PackageInfo
pub struct FlatpakTask {
    pub uuid: String,

    /// The Flatpak operation of this task
    pub kind: FlatpakTaskKind,
    /// The Flatpak installation in which the operation is to be performed.
    pub installation: InstallationInfo,
    /// If `true`, this task is only simulated and no changes are made to the
    /// corresponding installation.
    pub dry_run: bool,

    /// A Flatpak ref. Needed for [FlatpakTaskKind::Install] operations.
    pub ref_: Option<String>,
    /// A Flatpak remote. Needed for [FlatpakTaskKind::Install] operations.
    pub remote: Option<RemoteInfo>,
    /// The path of a Flatpak ref file ([FlatpakTaskKind::InstallRefFile])
    /// or a Flatpak bundle file ([FlatpakTaskKind::InstallBundleFile])
    pub path: Option<String>,
    /// There are cases where it isn't possible to update an already installed
    /// ref directly, and the previously installed ref have to get
    /// uninstalled first. This can be the case when a ref gets installed
    /// from a different remote, and the GPG keys wouldn't match for example.
    pub uninstall_before_install: bool,
}

impl FlatpakTask {
    pub fn new_install(
        package: &PackageInfo,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Self {
        let installation = package.remote.installation.as_ref().unwrap().clone();

        Self {
            uuid: Uuid::new_v4().to_string(),
            kind: FlatpakTaskKind::Install,
            installation,
            dry_run,
            ref_: Some(package.ref_.clone()),
            remote: Some(package.remote.clone()),
            uninstall_before_install,
            ..Default::default()
        }
    }

    pub fn new_install_ref_file(
        installation: &InstallationInfo,
        path: &str,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            kind: FlatpakTaskKind::InstallRefFile,
            installation: installation.clone(),
            dry_run,
            path: Some(path.to_owned()),
            uninstall_before_install,
            ..Default::default()
        }
    }

    pub fn new_install_bundle_file(
        installation: &InstallationInfo,
        path: &str,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            kind: FlatpakTaskKind::InstallBundleFile,
            installation: installation.clone(),
            dry_run,
            path: Some(path.to_owned()),
            uninstall_before_install,
            ..Default::default()
        }
    }

    pub fn new_uninstall(installation: &InstallationInfo, remote: &RemoteInfo, ref_: &str) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            kind: FlatpakTaskKind::Uninstall,
            installation: installation.clone(),
            dry_run: false,
            ref_: Some(ref_.to_owned()),
            remote: Some(remote.to_owned()),
            uninstall_before_install: false,
            ..Default::default()
        }
    }
}

impl From<FlatpakTask> for Task {
    fn from(flatpak_task: FlatpakTask) -> Self {
        Task {
            uuid: flatpak_task.uuid.clone(),
            cancellable: flatpak_task.kind != FlatpakTaskKind::Uninstall,
            kind: TaskKind::Flatpak(flatpak_task),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum FlatpakTaskKind {
    Install,
    InstallRefFile,
    InstallBundleFile,
    Uninstall,
    Update,
    UpdateInstallation,
    #[default]
    None,
}

impl FlatpakTaskKind {
    pub fn targets_single_package(&self) -> bool {
        self == &Self::Install
            || self == &Self::InstallRefFile
            || self == &Self::InstallBundleFile
            || self == &Self::Uninstall
            || self == &Self::Update
    }
}
