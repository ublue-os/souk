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
use zbus::zvariant::{Optional, Type};

use crate::shared::info::{InstallationInfo, PackageInfo, RemoteInfo};
use crate::shared::task::Task;

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
// TODO: This could be simplified by using PackageInfo
pub struct FlatpakTask {
    /// The Flatpak operation of this task
    pub operation_type: FlatpakOperationType,
    /// The Flatpak installation in which the operation is to be performed.
    pub installation: InstallationInfo,
    /// If `true`, this task is only simulated and no changes are made to the
    /// corresponding installation.
    pub dry_run: bool,

    /// A Flatpak ref. Needed for [FlatpakOperationType::Install] operations.
    pub ref_: Optional<String>,
    /// A Flatpak remote. Needed for [FlatpakOperationType::Install] operations.
    pub remote: Optional<RemoteInfo>,
    /// The path of a Flatpak ref file ([FlatpakOperationType::InstallRefFile])
    /// or a Flatpak bundle file ([FlatpakOperationType::InstallBundleFile])
    pub path: Optional<String>,
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
    ) -> Task {
        let installation = package.remote.installation.as_ref().unwrap().clone();

        let flatpak_task = Self {
            operation_type: FlatpakOperationType::Install,
            installation,
            dry_run,
            ref_: Some(package.ref_.clone()).into(),
            remote: Some(package.remote.clone()).into(),
            uninstall_before_install,
            ..Default::default()
        };

        Task::new_flatpak(flatpak_task, true)
    }

    pub fn new_install_ref_file(
        installation: &InstallationInfo,
        path: &str,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Task {
        let flatpak_task = Self {
            operation_type: FlatpakOperationType::InstallRefFile,
            installation: installation.clone(),
            dry_run,
            path: Some(path.to_owned()).into(),
            uninstall_before_install,
            ..Default::default()
        };

        Task::new_flatpak(flatpak_task, true)
    }

    pub fn new_install_bundle_file(
        installation: &InstallationInfo,
        path: &str,
        uninstall_before_install: bool,
        dry_run: bool,
    ) -> Task {
        let flatpak_task = Self {
            operation_type: FlatpakOperationType::InstallBundleFile,
            installation: installation.clone(),
            dry_run,
            path: Some(path.to_owned()).into(),
            uninstall_before_install,
            ..Default::default()
        };

        Task::new_flatpak(flatpak_task, true)
    }

    pub fn new_uninstall(installation: &InstallationInfo, remote: &RemoteInfo, ref_: &str) -> Task {
        let flatpak_task = Self {
            operation_type: FlatpakOperationType::Install,
            installation: installation.clone(),
            dry_run: false,
            ref_: Some(ref_.to_owned()).into(),
            remote: Some(remote.to_owned()).into(),
            uninstall_before_install: false,
            ..Default::default()
        };

        Task::new_flatpak(flatpak_task, false)
    }
}

impl Default for FlatpakTask {
    fn default() -> Self {
        Self {
            operation_type: FlatpakOperationType::default(),
            installation: InstallationInfo::default(),
            dry_run: false,
            ref_: None.into(),
            remote: None.into(),
            path: None.into(),
            uninstall_before_install: false,
        }
    }
}

#[derive(Default, Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub enum FlatpakOperationType {
    Install,
    InstallRefFile,
    InstallBundleFile,
    Uninstall,
    Update,
    UpdateInstallation,
    #[default]
    None,
}

