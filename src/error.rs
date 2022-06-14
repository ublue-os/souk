// Souk - error.rs
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

use thiserror::Error;

use crate::worker::WorkerError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Souk worker error")]
    Worker(#[from] WorkerError),

    #[error("Unsupported sideload type")]
    UnsupportedSideloadType,

    #[error("GLib error")]
    GLib(#[from] gtk::glib::Error),

    #[error("ZBus error")]
    ZBus(#[from] zbus::Error),
}

impl Error {
    pub fn message(&self) -> String {
        match self {
            Self::Worker(err) => err.message(),
            Self::GLib(err) => err.message().to_string(),
            _ => self.to_string(),
        }
    }
}
