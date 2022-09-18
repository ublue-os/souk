// Souk - mod.rs
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

pub mod context;

/// Consumes the data of the `souk-worker` process, and wraps them into usable
/// types so that they can easily consumed by the user interface (eg. gobjects
/// with properties)
mod flatpak;

/// The user interface
mod ui;

mod error;
mod i18n;

mod app;
pub use app::SkApplication;
