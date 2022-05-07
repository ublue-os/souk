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

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
//#[macro_use]
// extern crate serde_derive;
//#[macro_use]
// extern crate diesel;
//#[macro_use]
// extern crate diesel_migrations;
//#[macro_use]
// extern crate strum_macros;
#[macro_use]
extern crate gtk_macros;

pub mod flatpak;
pub mod ui;
pub mod worker;

mod app;
#[rustfmt::skip]
pub mod config;
pub mod i18n;
pub mod path;

pub use app::SkApplication;
