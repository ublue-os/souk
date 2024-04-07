// Souk - utils.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
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

use gio::Cancellable;
use gtk::gio;
use xb::prelude::*;

use crate::shared::path;

/// Check if a appstream xmlb silo exists at all (it doesn't have to be up to
/// date)
pub fn check_appstream_silo_exists() -> bool {
    let xmlb = gio::File::for_path(path::APPSTREAM_CACHE.clone());
    let silo = xb::Silo::new();

    if silo
        .load_from_file(&xmlb, xb::SiloLoadFlags::NONE, Cancellable::NONE)
        .is_ok()
    {
        // Ensure that silo contains plausible data and isn't entirely empty
        if let Ok(result) = silo.query("components", 1) {
            if !result.is_empty() {
                return true;
            }
        }
    }

    false
}
