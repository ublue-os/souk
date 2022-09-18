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

//! An easy to use Flatpak frontend
//!
//! Souk consists of several separate binaries that communicate via DBus.
//! The individual binaries are clearly separated into corresponding modules
//! ([`main`] and [`worker`])

// TODO: Add more detailed documentation

#![doc(
    html_logo_url = "https://gitlab.gnome.org/haecker-felix/souk/-/raw/main/data/icons/hicolor/scalable/apps/de.haeckerfelix.Souk.svg",
    html_favicon_url = "https://gitlab.gnome.org/haecker-felix/souk/-/raw/main/data/icons/hicolor/symbolic/apps/de.haeckerfelix.Souk-symbolic.svg"
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate gtk_macros;

/// Graphical user interface with relevant Flatpak components
pub mod main;

/// Components which are used / shared by all binaries
pub mod shared;

/// Performs Flatpak transactions or resource intensive activities
pub mod worker;
