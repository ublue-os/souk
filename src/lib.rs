// Souk - lib.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

#![doc(
    html_logo_url = "https://gitlab.gnome.org/haecker-felix/souk/-/raw/main/data/icons/hicolor/scalable/apps/de.haeckerfelix.Souk.svg",
    html_favicon_url = "https://gitlab.gnome.org/haecker-felix/souk/-/raw/main/data/icons/hicolor/symbolic/apps/de.haeckerfelix.Souk-symbolic.svg"
)]

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate gtk_macros;
#[macro_use]
extern crate lazy_static;

/// Consumes the data of the `souk-worker` process, and wraps them into usable
/// types so that they can easily consumed by the user interface (eg. gobjects
/// with properties)
pub mod flatpak;
/// The user interface
pub mod ui;
/// Components of the `souk-worker` binary, which does the actual Flatpak and
/// Appstream work / processing, and communicates via DBus with the main `souk`
/// process.
pub mod worker;

mod app;
mod error;
#[rustfmt::skip]
pub mod config;
pub mod i18n;
pub mod path;

pub use app::SkApplication;
