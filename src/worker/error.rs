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

use gtk::gio::IOErrorEnum;
use gtk::glib::Error;
use zbus::DBusError;

#[derive(DBusError, Debug, PartialEq)]
pub enum WorkerError {
    Network(String),
    IO(String),

    DryRunRuntimeNotFound(String),

    GLibCancelled,
    GLib(String),

    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
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

impl From<Error> for WorkerError {
    fn from(item: Error) -> Self {
        if item.kind::<IOErrorEnum>() == Some(IOErrorEnum::Cancelled) {
            return Self::GLibCancelled;
        }

        Self::GLib(item.message().to_string())
    }
}

impl WorkerError {
    pub fn message(&self) -> String {
        match self {
            Self::Network(message) => message.into(),
            Self::IO(message) => message.into(),
            Self::DryRunRuntimeNotFound(runtime) => {
                format!("Unable to find required runtime {}", runtime)
            }
            Self::GLibCancelled => "The operation got cancelled.".into(),
            Self::GLib(message) => message.into(),
            Self::ZBus(err) => err.to_string(),
        }
    }
}
