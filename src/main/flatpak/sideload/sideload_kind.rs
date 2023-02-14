// Souk - sideload_kind.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use gio::prelude::*;
use gio::File;
use glib::Enum;
use gtk::{gio, glib};

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkSideloadKind")]
pub enum SkSideloadKind {
    Ref,
    Repo,
    Bundle,
    None,
}

impl SkSideloadKind {
    pub fn determine_type(file: &File) -> SkSideloadKind {
        let file = file.path().unwrap();

        match file.extension().unwrap_or_default().to_str().unwrap() {
            "flatpakref" => SkSideloadKind::Ref,
            "flatpakrepo" => SkSideloadKind::Repo,
            "flatpak" => SkSideloadKind::Bundle,
            _ => SkSideloadKind::None,
        }
    }
}
