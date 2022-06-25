// Souk - utils.rs
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

use gtk::glib;

pub fn size_to_markup(size: &str) -> String {
    let size: u64 = size.parse().unwrap();
    let formatted = glib::format_size(size).to_string();

    let spl: Vec<&str> = formatted.split('\u{a0}').collect();
    format!("{}\u{a0}<small>{}</small>", spl[0], spl[1])
}
