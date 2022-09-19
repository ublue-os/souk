// Souk - app.rs
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

use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::shared::config;
use crate::worker::{dbus_server, TransactionManager};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkWorkerApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorkerApplication {
        const NAME: &'static str = "SkWorkerApplication";
        type ParentType = adw::Application;
        type Type = super::SkWorkerApplication;
    }

    impl ObjectImpl for SkWorkerApplication {}

    impl GtkApplicationImpl for SkWorkerApplication {}

    impl AdwApplicationImpl for SkWorkerApplication {}

    impl ApplicationImpl for SkWorkerApplication {
        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);

            debug!("Application -> startup");

            // TODO: replace with proper task scheduling
            async_std::task::block_on(async {
                Self::Type::spawn_dbus_server().await;
            });
        }
    }
}

glib::wrapper! {
    pub struct SkWorkerApplication(ObjectSubclass<imp::SkWorkerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SkWorkerApplication {
    pub fn run() {
        // We need to use a different app id for the gio application, otherwise zbus
        // wouldn't be able to use it any more. See https://gitlab.gnome.org/GNOME/glib/-/issues/2756
        let app_id = format!("{}.App", config::WORKER_APP_ID);

        debug!(
            "{} Worker ({}) ({}) - Version {} ({})",
            config::NAME,
            app_id,
            config::VCS_TAG,
            config::VERSION,
            config::PROFILE
        );

        let app = glib::Object::new::<SkWorkerApplication>(&[
            ("application-id", &Some(app_id)),
            ("flags", &gio::ApplicationFlags::IS_SERVICE),
        ])
        .unwrap();

        // Start running gtk::Application
        app.run();
    }

    async fn spawn_dbus_server() {
        use async_std::channel::unbounded;
        debug!("Start souk-worker dbus server...");

        // The Flatpak transaction manager is multithreaded, because Flatpak
        // transactions are blocking. Therefore it uses message passing for inter
        // thread communication
        let (command_sender, command_receiver) = unbounded();
        let (message_sender, message_receiver) = unbounded();
        TransactionManager::start(message_sender, command_receiver);

        dbus_server::start(command_sender, message_receiver)
            .await
            .unwrap();
    }
}

impl Default for SkWorkerApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
