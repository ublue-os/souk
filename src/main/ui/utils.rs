// Souk - utils.rs
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

use gtk::glib;
use gtk::prelude::*;

pub fn size_to_markup(size: &str) -> String {
    if let Ok(size) = size.parse::<u64>() {
        let formatted = glib::format_size(size).to_string();

        let mut spl: Vec<&str> = formatted.split('\u{a0}').collect();
        if spl.len() == 1 {
            spl = formatted.split(' ').collect();
        }

        if spl.len() == 2 {
            format!("{}\u{a0}<small>{}</small>", spl[0], spl[1])
        } else {
            warn!("Unable to build size markup: {}", size);
            String::new()
        }
    } else {
        size.to_string()
    }
}

pub fn remove_css_colors<T: IsA<gtk::Widget>>(widget: &T) {
    let css_classes = vec![
        "color-neutral",
        "color-green",
        "color-blue",
        "color-orange",
        "color-yellow",
        "color-red",
    ];

    for class in &css_classes {
        widget.remove_css_class(class);
    }
}

pub fn clear_box(box_: &gtk::Box) {
    while let Some(child) = box_.first_child() {
        box_.remove(&child);
    }
}
