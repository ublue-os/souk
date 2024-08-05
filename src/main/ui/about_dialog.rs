// Souk - about_window.rs
// Copyright (C) 2021-2024  Felix Häcker <haeckerfelix@gnome.org>
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

use adw::prelude::*;

use crate::main::i18n::i18n;
use crate::main::ui::main::SkApplicationWindow;
use crate::shared::config;

pub fn show(parent: &SkApplicationWindow) {
    let dialog = adw::AboutDialog::from_appdata(
        &format!("{}/metainfo.xml", config::PATH_ID),
        Some(config::VERSION),
    );

    let _version = match config::PROFILE {
        "development" => {
            dialog.set_debug_info(&format!("Git Commit: {}", config::VCS_TAG));
            format!("{}-devel", config::VERSION)
        }
        _ => config::VERSION.to_string(),
    };

    dialog.set_version(config::VERSION);
    dialog.set_developers(&[
        "Christopher Davis <christopherdavis@gnome.org>",
        "Felix Häcker <haeckerfelix@gnome.org>",
        "Maximiliano Sandoval <msandova@gnome.org>",
        "Zander Brown <zbrown@gnome.org>",
    ]);
    dialog.set_designers(&["Tobias Bernard"]);
    dialog.set_translator_credits(&i18n("translator-credits"));

    dialog.present(Some(parent));
}
