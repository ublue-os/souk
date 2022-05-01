// Souk - dbus.rs
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

use zbus::{dbus_interface, ConnectionBuilder, Result};

use crate::config;

struct Worker;

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl Worker {
    async fn install_flatpak_bundle(&self, path: &str) {
        info!("Installing Flatpak Bundle: {}", path);
    }
}

pub async fn start_server() -> Result<()> {
    let name = format!("{}.Worker", config::APP_ID);
    let worker = Worker {};

    let _ = ConnectionBuilder::session()?
        .name(name)?
        .serve_at("/de/haeckerfelix/Souk/Worker", worker)?
        .build()
        .await?;

    Ok(())
}
