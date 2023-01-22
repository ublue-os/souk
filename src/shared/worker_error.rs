// Souk - worker_error.rs
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

use std::fmt;

use gtk::gio::IOErrorEnum;
use gtk::glib;
use gtk::glib::Error as GLibError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zbus::zvariant::Type;

#[derive(Deserialize, Clone, Serialize, Type, Error, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "WorkerError")]
pub enum WorkerError {
    IO(String),
    // The string is unused. Unfortunately we need it, so the struct can be (de)serialized with
    // dbus/zbus
    GLibCancelled(String),
    GLib(String),
    DryRunRuntimeNotFound(String),
}

impl Default for WorkerError {
    fn default() -> Self {
        Self::GLib(String::new())
    }
}

impl From<std::io::Error> for WorkerError {
    fn from(item: std::io::Error) -> Self {
        Self::IO(item.to_string())
    }
}

impl From<isahc::Error> for WorkerError {
    fn from(item: isahc::Error) -> Self {
        Self::IO(item.to_string())
    }
}

impl From<GLibError> for WorkerError {
    fn from(item: GLibError) -> Self {
        if item.kind::<IOErrorEnum>() == Some(IOErrorEnum::Cancelled) {
            return Self::GLibCancelled(String::new());
        }

        Self::GLib(item.message().to_string())
    }
}

impl fmt::Display for WorkerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let msg = match self {
            Self::IO(message) => message.into(),
            Self::DryRunRuntimeNotFound(runtime) => {
                format!("Unable to find required runtime {runtime}")
            }
            Self::GLibCancelled(_) => "The operation got cancelled.".into(),
            Self::GLib(message) => message.into(),
        };
        write!(fmt, "{msg}")
    }
}
