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

use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use zbus::{dbus_interface, ConnectionBuilder, Result, SignalContext};

use crate::config;
use crate::worker::flatpak::{Command, Response};

#[derive(Debug)]
struct Worker {
    sender: Sender<Command>,
}

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl Worker {
    async fn install_flatpak_bundle(&self, path: &str) {
        self.sender
            .send(Command::InstallFlatpakBundle(path.to_string()))
            .await
            .unwrap();
    }

    #[dbus_interface(signal)]
    async fn progress(signal_ctxt: &SignalContext<'_>, progress: i32) -> zbus::Result<()>;
}

pub async fn start(sender: Sender<Command>, mut receiver: Receiver<Response>) -> Result<()> {
    let name = format!("{}.Worker", config::APP_ID);
    let path = "/de/haeckerfelix/Souk/Worker";
    let worker = Worker { sender };

    let con = ConnectionBuilder::session()?
        .name(name)?
        .serve_at(path, worker)?
        .build()
        .await?;

    let signal_ctxt = SignalContext::new(&con, path).unwrap();
    while let Some(response) = receiver.next().await {
        Worker::progress(&signal_ctxt, response.progress)
            .await
            .unwrap();
    }

    Ok(())
}
