// Souk - about_dialog.rs
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

use gtk::prelude::*;

use crate::config;
use crate::i18n::*;
use crate::ui::SkApplicationWindow;

pub fn show_about_dialog(window: &SkApplicationWindow) {
    let vcs_tag = config::VCS_TAG;
    let version: String = match config::PROFILE {
        "development" => format!("{} \n(Development Commit {})", config::VERSION, vcs_tag),
        _ => format!("{}-stable", config::VERSION),
    };

    let dialog = gtk::AboutDialog::new();
    dialog.set_program_name(Some(config::NAME));
    dialog.set_logo_icon_name(Some(config::APP_ID));
    dialog.set_comments(Some(&i18n("Flatpak App Store")));
    dialog.set_copyright(Some("© 2021 Felix Häcker"));
    dialog.set_website(Some("https://gitlab.gnome.org/haecker-felix/souk"));
    dialog.set_translator_credits(Some(&i18n("translator-credits")));
    dialog.set_license_type(gtk::License::Gpl30);
    dialog.set_version(Some(version.as_str()));
    dialog.set_transient_for(Some(window));
    dialog.set_modal(true);

    dialog.set_authors(&["Felix Häcker"]);
    dialog.set_artists(&["Tobias Bernard"]);

    dialog.show();
}
