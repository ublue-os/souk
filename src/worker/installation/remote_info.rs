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
use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

#[derive(Deserialize, Serialize, Type, Debug, Clone, Eq, PartialEq)]
pub struct RemoteInfo {
    pub name: String,
    pub repository_url: String,

    // Metadata which is in the .flatpakrepo file
    pub title: Optional<String>,
    pub description: Optional<String>,
    pub comment: Optional<String>,
    pub homepage: Optional<String>,
    pub icon: Optional<String>,
}

impl RemoteInfo {
    pub fn new(name: &str, repository_url: &str) -> Self {
        Self {
            name: name.into(),
            repository_url: repository_url.into(),
            ..Default::default()
        }
    }

    pub fn flatpak_remote(&self) -> Remote {
        let remote = Remote::new(&self.name);
        remote.set_url(&self.repository_url);

        if let Some(value) = self.title.as_ref() {
            remote.set_title(value);
        }

        if let Some(value) = self.description.as_ref() {
            remote.set_description(value);
        }

        if let Some(value) = self.comment.as_ref() {
            remote.set_comment(value);
        }

        if let Some(value) = self.homepage.as_ref() {
            remote.set_homepage(value);
        }

        if let Some(value) = self.icon.as_ref() {
            remote.set_icon(value);
        }

        remote
    }

    pub fn set_flatpak_remote(&mut self, remote: &Remote) {
        if let Some(value) = remote.title() {
            self.title = Some(value.into()).into();
        }
        if let Some(value) = remote.description() {
            self.description = Some(value.into()).into();
        }
        if let Some(value) = remote.comment() {
            self.comment = Some(value.into()).into();
        }
        if let Some(value) = remote.homepage() {
            self.homepage = Some(value.into()).into();
        }
        if let Some(value) = remote.icon() {
            self.icon = Some(value.into()).into();
        }
    }
}

impl Default for RemoteInfo {
    fn default() -> Self {
        Self {
            name: String::default(),
            repository_url: String::default(),

            title: None.into(),
            description: None.into(),
            comment: None.into(),
            homepage: None.into(),
            icon: None.into(),
        }
    }
}
