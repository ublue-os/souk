// Souk - souk-worker.rs
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

use std::env;

use gtk::glib;
use souk::shared::{config, path};
use souk::worker::SkWorkerApplication;

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // We need to overwrite `FLATPAK_BINARY`, otherwise the exported files (eg.
    // desktop or DBus services) would have the wrong path ("/app/bin/flatpak").
    //
    // This only applies to `user` installations, since `system` operations are
    // getting handled by Flatpak SystemHelper on host side.
    env::set_var("FLATPAK_BINARY", "/usr/bin/flatpak");
    env::set_var("FLATPAK_BWRAP", "/app/bin/flatpak-bwrap");

    // Initialize paths
    path::init().expect("Unable to create paths.");

    // Initialize variables
    glib::set_application_name(config::NAME);

    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire().unwrap();

    // Run app itself
    SkWorkerApplication::run();
}
