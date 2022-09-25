// Souk - worker.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
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

use std::sync::Arc;

use async_std::channel::Sender;
use glib::Downgrade;
use gtk::glib;

use crate::shared::task::{AppstreamTask, Response};

#[derive(Debug, Clone, Downgrade)]
pub struct AppstreamWorker {
    sender: Arc<Sender<Response>>,
}

impl AppstreamWorker {
    pub fn new(sender: Sender<Response>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn process_task(&self, _task: AppstreamTask, _task_uuid: &str) {
        unimplemented!()
    }
}
