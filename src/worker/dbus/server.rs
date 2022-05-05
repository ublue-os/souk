// Souk - server.rs
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

use glib::clone;
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{Installation, Transaction};
use zbus::{dbus_interface, ConnectionBuilder, Result};

use crate::config;

struct Worker;

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl Worker {
    async fn install_flatpak_bundle(&self, path: &str) {
        info!("Installing Flatpak Bundle: {}", path);

        let system_installation = Installation::new_system(gio::Cancellable::NONE).unwrap();

        let transaction =
            Transaction::for_installation(&system_installation, gio::Cancellable::NONE).unwrap();

        let file = gio::File::for_parse_name(path);
        transaction.add_install_bundle(&file, None).unwrap();

        transaction.connect_new_operation(|_transaction, _operation, progress| {
            info!("Transaction new operation");
            progress.set_update_frequency(10);

            let duration = std::time::Duration::from_secs(1);
            glib::timeout_add_local(
                duration,
                clone!(@strong progress => @default-return glib::Continue(false), move ||{
                    if progress.progress() == 100{
                        glib::Continue(false)
                    }else{
                        debug!("Progress: {}", progress.progress());
                        glib::Continue(true)
                    }
                }),
            );
        });

        transaction.connect_add_new_remote(|_transaction, _reason, _s1, _s2, _s3| {
            info!("Transaction add new remote");
            true
        });

        transaction.run(gio::Cancellable::NONE).unwrap();
    }
}

pub async fn start() -> Result<()> {
    let name = format!("{}.Worker", config::APP_ID);
    let worker = Worker {};

    let _ = ConnectionBuilder::session()?
        .name(name)?
        .serve_at("/de/haeckerfelix/Souk/Worker", worker)?
        .build()
        .await?;

    Ok(())
}
