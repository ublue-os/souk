// Souk - remote_info.rs
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

use flatpak::prelude::*;
use flatpak::Remote;
use gtk::glib::Error;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Hash, Type, Debug, Clone, Eq, PartialEq, Default)]
pub struct RemoteInfo {
    pub name: String,
    pub repository_url: String,

    gpg_key: String,

    // Optional metadata from .flatpakrepo file
    pub title: String,
    pub description: String,
    pub comment: String,
    pub homepage: String,
    pub icon: String,
}

impl RemoteInfo {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.into(),
            repository_url: url.into(),
            ..Default::default()
        }
    }

    pub fn set_gpg_key(&mut self, key: &str) {
        self.gpg_key = key.into();
    }
}

impl From<&Remote> for RemoteInfo {
    fn from(remote: &Remote) -> Self {
        let mut info = Self {
            name: remote.name().unwrap().into(),
            repository_url: remote.url().unwrap().into(),
            ..Default::default()
        };

        if let Some(value) = remote.title() {
            info.title = value.into();
        }

        if let Some(value) = remote.description() {
            info.description = value.into();
        }

        if let Some(value) = remote.comment() {
            info.comment = value.into();
        }

        if let Some(value) = remote.homepage() {
            info.homepage = value.into();
        }

        if let Some(value) = remote.icon() {
            info.icon = value.into();
        }

        info
    }
}

impl TryInto<Remote> for RemoteInfo {
    type Error = Error;

    fn try_into(self) -> Result<Remote, Self::Error> {
        let remote = Remote::new(&self.name);
        remote.set_url(&self.repository_url);

        if self.gpg_key.is_empty() {
            return Err(Error::new(
                flatpak::Error::Untrusted,
                "Can't add remote object without gpg key.",
            ));
        }

        if !self.title.is_empty() {
            remote.set_title(&self.title);
        }

        if !self.description.is_empty() {
            remote.set_title(&self.description);
        }

        if !self.comment.is_empty() {
            remote.set_title(&self.comment);
        }

        if !self.homepage.is_empty() {
            remote.set_title(&self.homepage);
        }

        if !self.icon.is_empty() {
            remote.set_title(&self.icon);
        }

        Ok(remote)
    }
}
