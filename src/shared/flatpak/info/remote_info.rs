// Souk - remote_info.rs
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

use derivative::Derivative;
use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use gtk::glib;
use gtk::glib::Error;
use serde::{Deserialize, Serialize};

use super::InstallationInfo;
use crate::shared::WorkerError;

#[derive(Default, Derivative, Deserialize, Serialize, Hash, Clone, Eq, PartialEq, glib::Boxed)]
#[boxed_type(name = "RemoteInfo", nullable)]
#[derivative(Debug)]
pub struct RemoteInfo {
    pub name: String,
    pub repository_url: String,
    pub installation: Option<InstallationInfo>,

    #[derivative(Debug = "ignore")]
    repo_bytes: Option<Vec<u8>>,
}

impl RemoteInfo {
    pub fn new(
        name: String,
        repository_url: String,
        installation: Option<InstallationInfo>,
    ) -> Self {
        Self {
            name,
            repository_url,
            installation,
            ..Default::default()
        }
    }

    pub fn from_flatpak(remote: &Remote, installation: &Installation) -> Self {
        Self::new(
            remote.name().unwrap().into(),
            remote.url().unwrap_or_default().into(),
            Some(installation.into()),
        )
    }

    pub fn from_repo_file(name: &str, bytes: Vec<u8>) -> Result<Self, WorkerError> {
        let g_bytes = glib::Bytes::from(&bytes);

        // Verify that bytes can be read into a Flatpak `Remote` object
        let remote = Remote::from_file(name, &g_bytes)?;

        Ok(Self {
            name: remote.name().unwrap().into(),
            repository_url: remote.url().unwrap_or_default().into(),
            repo_bytes: Some(bytes),
            ..Default::default()
        })
    }

    pub fn set_repo_bytes(&mut self, bytes: Vec<u8>) {
        self.repo_bytes = Some(bytes.to_vec());
    }
}

impl TryInto<Remote> for RemoteInfo {
    type Error = Error;

    fn try_into(self) -> Result<Remote, Self::Error> {
        if let Some(bytes) = self.repo_bytes.as_ref() {
            let g_bytes = glib::Bytes::from(bytes);
            let r = Remote::from_file(&self.name, &g_bytes)?;
            r.set_gpg_verify(true);
            Ok(r)
        } else {
            Err(Error::new(
                flatpak::Error::RemoteNotFound,
                "Unable to create Flatpak remote object, no repodata available.",
            ))
        }
    }
}
