// Souk - souk-worker.rs
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

use std::env;
use std::path::Path;
use std::process::Command;

use gtk::glib;
use souk::shared::{config, path};
use souk::worker::SkWorkerApplication;

fn main() -> glib::ExitCode {
    // Initialize logger
    pretty_env_logger::init();

    // Check if Souk gets executed as Flatpak
    let flatpak_info = Path::new("/.flatpak-info");
    if flatpak_info.exists() {
        log::debug!("Running in Flatpak sandbox, overwriting environment variables...");
        // We need to overwrite `FLATPAK_BINARY`, otherwise the exported files (eg.
        // desktop or DBus services) would have the wrong path ("/app/bin/flatpak").
        //
        // This only applies to `user` installations, since `system` operations are
        // getting handled by Flatpak SystemHelper on host side.
        set_env_var("FLATPAK_BINARY", "/usr/bin/flatpak");
        set_env_var("FLATPAK_BWRAP", "/app/bin/flatpak-bwrap");

        // Mirror language / locale env variables from host side. We need to set these
        // so that libflatpak correctly detects the languages, and correctly installs
        // the `.Locale` refs. If we would not do this, then on the host side during a
        // "flatpak update" the `.Locale` refs would be installed / updated again with
        // the appropriate translation.
        let envs = vec![
            "LANG",
            "LANGUAGE",
            "LC_ALL",
            "LC_MESSAGES",
            "LC_ADDRESS",
            "LC_COLLATE",
            "LC_CTYPE",
            "LC_IDENTIFICATION",
            "LC_MONETARY",
            "LC_MEASUREMENT",
            "LC_NAME",
            "LC_NUMERIC",
            "LC_PAPER",
            "LC_TELEPHONE",
            "LC_TIME",
        ];

        for env in envs {
            if let Some(var) = retrieve_host_env(env) {
                set_env_var(env, var.trim());
            }
        }
    }

    // Initialize paths
    path::init().expect("Unable to create paths.");

    // Initialize variables
    glib::set_application_name(config::NAME);

    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire().unwrap();

    // Run app itself
    SkWorkerApplication::run()
}

fn retrieve_host_env(env: &str) -> Option<String> {
    if let Ok(output) = Command::new("flatpak-spawn")
        // Messes up flatpak-spawn output with debug prints
        .env_remove("G_MESSAGES_DEBUG")
        .arg("--host")
        .arg("printenv")
        .arg(env)
        .output()
    {
        return String::from_utf8(output.stdout).ok();
    }

    None
}

fn set_env_var(env: &str, var: &str) {
    log::debug!("Set env var \"{env}\" to \"{var}\"");
    env::set_var(env, var);
}
