// Souk - dbus_proxy.rs
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

use crate::shared::config;
use crate::shared::task::{Task, TaskResponse as TaskResponse_};

#[zbus::dbus_proxy(interface = "de.haeckerfelix.Souk.Worker1")]
trait Worker {
    fn run_task(&self, task: Task) -> zbus::Result<()>;

    fn cancel_task(&self, task: Task) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn task_response(&self, task_response: TaskResponse_) -> zbus::Result<()>;
}

impl Default for WorkerProxy<'static> {
    fn default() -> Self {
        let fut = async {
            let session = zbus::Connection::session().await?;
            let name = format!("{}.Worker", config::APP_ID);

            WorkerProxy::builder(&session)
                .destination(name)?
                // TODO: Don't hardcode path here
                .path(config::DBUS_PATH)?
                .build()
                .await
        };

        async_std::task::block_on(fut).unwrap()
    }
}
