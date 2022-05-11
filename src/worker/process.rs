// Souk - process.rs
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

use std::cell::RefCell;
use std::fs;

use async_std::process::{Child, Command};

use crate::path;

#[derive(Debug, Default)]
pub struct Process {
    child: RefCell<Option<Child>>,
}

impl Process {
    /// Spawn `souk-worker` binary outside of the Flatpak sandbox.
    /// This method gets called from the `souk` binary.
    pub fn spawn(&self) {
        debug!("Start souk-worker process...");

        // First copy worker binary outside of sandbox
        let mut destination = path::BIN.clone();
        destination.push("souk-worker");
        fs::copy("/app/bin/souk-worker", destination).expect("Unable to copy souk-worker binary");

        // If we kill flatpak-spawn, we also want to kill the child process too.
        let mut args: Vec<String> = vec!["--watch-bus".into()];

        // We cannot do stuff inside the Flatpak Sandbox,
        // so we have to spawn the worker process on host side
        args.push("--host".into());
        args.push("souk-worker".into());

        let child = Command::new("flatpak-spawn").args(&args).spawn().unwrap();
        *self.child.borrow_mut() = Some(child);
    }

    pub fn kill(&self) {
        debug!("Kill souk-worker process...");
        if let Some(mut child) = self.child.borrow_mut().take() {
            child.kill().expect("Unable to kill souk-worker process");
        } else {
            debug!("souk-worker is not running!");
        }
    }
}
