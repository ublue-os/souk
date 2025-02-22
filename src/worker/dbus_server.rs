// Souk - dbus_server.rs
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

use async_std::channel::Sender;
use zbus::SignalContext;

use crate::shared::task::Task;

#[derive(Debug)]
pub struct WorkerServer {
    pub task_sender: Sender<Task>,
    pub cancel_sender: Sender<Task>,
}

#[zbus::interface(name = "de.haeckerfelix.Souk.Worker1")]
impl WorkerServer {
    async fn run_task(&self, task_json: &str) {
        match serde_json::from_str(task_json) {
            Ok(task) => self.task_sender.send(task).await.unwrap(),
            Err(err) => error!(
                "Unable to run task, deserialization failed: {}",
                err.to_string()
            ),
        }
    }

    async fn cancel_task(&self, task_json: &str) {
        match serde_json::from_str(task_json) {
            Ok(task) => self.cancel_sender.send(task).await.unwrap(),
            Err(err) => error!(
                "Unable to cancel task, deserialization failed: {}",
                err.to_string()
            ),
        }
    }

    #[zbus(signal)]
    pub async fn task_response(
        signal_ctxt: &SignalContext<'_>,
        task_response_json: &str,
    ) -> zbus::Result<()>;
}
