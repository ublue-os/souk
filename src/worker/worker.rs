// Souk - worker.rs
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

use gtk::glib;
use gtk::subclass::prelude::*;
use zbus::Result;
use futures_util::stream::StreamExt;

use crate::worker::dbus::WorkerProxy;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorker {
        pub proxy: WorkerProxy<'static>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorker {
        const NAME: &'static str = "SkWorker";
        type ParentType = glib::Object;
        type Type = super::SkWorker;
    }

    impl ObjectImpl for SkWorker {}
}

glib::wrapper! {
    pub struct SkWorker(ObjectSubclass<imp::SkWorker>);
}

impl SkWorker {
    pub async fn install_flatpak_bundle(&self, path: &str) -> Result<()> {
        self.imp().proxy.install_flatpak_bundle(path).await?;

        let mut progress = self.imp().proxy.receive_progress().await?;
        while let Some(progress) = progress.next().await{
            dbg!(progress.args().unwrap().progress);
        }

        Ok(())
    }
}

impl Default for SkWorker {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }
}
