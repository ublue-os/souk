// Souk - response.rs
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

use serde::{Deserialize, Serialize};
use zbus::zvariant::{Optional, Type};

use crate::shared::task::{AppstreamResponse, FlatpakResponse};

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub struct Response {
    /// The UUID of the corresponding task
    pub uuid: String,
    pub type_: ResponseType,

    // This should have been an enum, unfortunately not supported by zbus / dbus
    flatpak_response: Optional<FlatpakResponse>,
    appstream_response: Optional<AppstreamResponse>,
    error_response: Optional<String>,
}

impl Response {
    pub fn new_done(uuid: String) -> Self {
        Self {
            uuid,
            type_: ResponseType::Done,
            flatpak_response: None.into(),
            appstream_response: None.into(),
            error_response: None.into(),
        }
    }

    pub fn new_cancelled(uuid: String) -> Self {
        Self {
            uuid,
            type_: ResponseType::Cancelled,
            flatpak_response: None.into(),
            appstream_response: None.into(),
            error_response: None.into(),
        }
    }

    pub fn new_error(uuid: String, error: String) -> Self {
        Self {
            uuid,
            type_: ResponseType::Error,
            flatpak_response: None.into(),
            appstream_response: None.into(),
            error_response: Some(error).into(),
        }
    }

    pub fn new_flatpak(uuid: String, type_: ResponseType, response: FlatpakResponse) -> Self {
        Self {
            uuid,
            type_,
            flatpak_response: Some(response).into(),
            appstream_response: None.into(),
            error_response: None.into(),
        }
    }

    pub fn new_appstream(uuid: String, type_: ResponseType, response: AppstreamResponse) -> Self {
        Self {
            uuid,
            type_,
            flatpak_response: None.into(),
            appstream_response: Some(response).into(),
            error_response: None.into(),
        }
    }

    /// Returns [FlatpakResponse] if this is a Flatpak response.
    pub fn flatpak_response(&self) -> Option<FlatpakResponse> {
        self.flatpak_response.clone().into()
    }

    /// Returns [AppstreamResponse] if this is a Flatpak response
    pub fn appstream_response(&self) -> Option<FlatpakResponse> {
        self.flatpak_response.clone().into()
    }

    /// Returns a [String] if this is a error response
    pub fn error_response(&self) -> Option<String> {
        self.error_response.clone().into()
    }
}

#[derive(Deserialize, Serialize, Type, Eq, PartialEq, Debug, Clone, Hash)]
pub enum ResponseType {
    /// Response contains updates / progress information of a running task.
    Update,
    /// Task successfully completed.
    Done,
    /// Task has been canceled (eg. by user).
    Cancelled,
    /// Task was terminated due to an error. See [Response] `error_response`
    /// field for reason / message.
    Error,
}
