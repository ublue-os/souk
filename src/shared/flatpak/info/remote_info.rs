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

use base64::engine::general_purpose;
use base64::Engine as _;
use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use gtk::glib;
use gtk::glib::Error;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use super::InstallationInfo;

#[derive(Deserialize, Serialize, Hash, Type, Debug, Clone, Eq, PartialEq, glib::Boxed)]
#[boxed_type(name = "RemoteInfo", nullable)]
pub struct RemoteInfo {
    pub name: String,
    pub repository_url: String,
    pub installation: Optional<InstallationInfo>,

    gpg_key: String,

    // Optional metadata from .flatpakrepo file
    pub title: String,
    pub description: String,
    pub comment: String,
    pub homepage: String,
    pub icon: String,
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
            installation: installation.into(),
            ..Default::default()
        }
    }

    pub fn from_flatpak(remote: &Remote, installation: Option<&Installation>) -> Self {
        let installation: Option<InstallationInfo> = installation.map(|i| i.into());

        let mut info = Self {
            name: remote.name().unwrap().into(),
            repository_url: remote.url().unwrap_or_default().into(),
            installation: installation.into(),
            ..Default::default()
        };
        info.update_metadata(remote);

        info
    }

    pub fn set_gpg_key(&mut self, key: &str) {
        self.gpg_key = key.into();
    }

    /// Updates the optional metadata fields using a Flatpak [Remote] object
    pub fn update_metadata(&mut self, remote: &Remote) {
        if let Some(value) = remote.title() {
            self.title = value.into();
        }

        if let Some(value) = remote.description() {
            self.description = value.into();
        }

        if let Some(value) = remote.comment() {
            self.comment = value.into();
        }

        if let Some(value) = remote.homepage() {
            self.homepage = value.into();
        }

        if let Some(value) = remote.icon() {
            self.icon = value.into();
        }
    }
}

impl Default for RemoteInfo {
    fn default() -> Self {
        Self {
            name: String::default(),
            repository_url: String::default(),
            installation: None.into(),
            gpg_key: String::default(),
            title: String::default(),
            description: String::default(),
            comment: String::default(),
            homepage: String::default(),
            icon: String::default(),
        }
    }
}

impl TryInto<Remote> for RemoteInfo {
    type Error = Error;

    fn try_into(self) -> Result<Remote, Self::Error> {
        if self.gpg_key.is_empty() {
            return Err(Error::new(
                flatpak::Error::Untrusted,
                "Can't create Flatpak remote object without gpg key.",
            ));
        }

        let remote = Remote::new(&self.name);
        remote.set_url(&self.repository_url);

        if let Ok(bytes) = general_purpose::STANDARD.decode(self.gpg_key) {
            remote.set_gpg_key(&glib::Bytes::from(&bytes));
        } else {
            return Err(Error::new(
                flatpak::Error::Untrusted,
                "Unable to retrieve GPG key.",
            ));
        }

        if !self.title.is_empty() {
            remote.set_title(&self.title);
        }

        if !self.description.is_empty() {
            remote.set_description(&self.description);
        }

        if !self.comment.is_empty() {
            remote.set_comment(&self.comment);
        }

        if !self.homepage.is_empty() {
            remote.set_homepage(&self.homepage);
        }

        if !self.icon.is_empty() {
            remote.set_icon(&self.icon);
        }

        Ok(remote)
    }
}
