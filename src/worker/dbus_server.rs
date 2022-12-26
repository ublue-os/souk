// Souk - dbus_server.rs
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

use async_std::channel::Sender;
use zbus::{dbus_interface, SignalContext};

use crate::shared::task::{Response, Task};

#[derive(Debug)]
pub struct WorkerServer {
    pub task_sender: Sender<Task>,
    pub cancel_sender: Sender<Task>,
}

#[dbus_interface(name = "de.haeckerfelix.Souk.Worker1")]
impl WorkerServer {
    async fn run_task(&self, task: Task) {
        self.task_sender.send(task).await.unwrap();
    }

    async fn cancel_task(&self, task: Task) {
        self.cancel_sender.send(task).await.unwrap();
    }

    #[dbus_interface(signal)]
    pub async fn task_response(
        signal_ctxt: &SignalContext<'_>,
        response: Response,
    ) -> zbus::Result<()>;
}
