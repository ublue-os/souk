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

use crate::i18n::i18n;

pub fn normalize_string(string: &str) -> String {
    string
        .chars()
        .flat_map(|x| {
            if x.is_alphanumeric() {
                x.to_lowercase()
            } else {
                '-'.to_lowercase()
            }
        })
        .collect()
}

pub fn runtime_ref_to_display_name(ref_: &str) -> String {
    if ref_.ends_with(".Platform") {
        return i18n("System Libraries and Services");
    }

    if ref_.ends_with(".Sdk") {
        return i18n("System Development Packages");
    }

    if ref_.ends_with(".Locale") {
        return i18n("Localization Data");
    }

    if ref_.ends_with(".Doc") {
        return i18n("Documentation Data");
    }

    if ref_.ends_with(".openh264") || ref_.ends_with(".ffmpeg-full") {
        return i18n("Multimedia Codecs");
    }

    if ref_.contains(".GL.") {
        return i18n("Graphic Drivers");
    }

    if ref_.contains(".GL32.") {
        return i18n("32-Bit Graphic Drivers");
    }

    if ref_.contains(".Compat.") {
        return i18n("32-Bit Compatibility Packages");
    }

    i18n("Shared System Package")
}
