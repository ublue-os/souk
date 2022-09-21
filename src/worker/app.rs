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

use std::cell::RefCell;
use std::time::Duration;

use adw::subclass::prelude::*;
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::prelude::*;
use futures::future::join_all;
use gio::subclass::prelude::ApplicationImpl;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use rusty_pool::ThreadPool;
use zbus::{Connection, ConnectionBuilder, SignalContext};

use crate::shared::config;
use crate::worker::dbus_server::WorkerServer;
use crate::worker::transaction::{FlatpakMessage, FlatpakTask};
use crate::worker::FlatpakWorker;

/// Specifies how many tasks can be executed in parallel
const WORKER_THREADS: usize = 4;
const DBUS_PATH: &str = "/de/haeckerfelix/Souk/Worker";

mod imp {
    use super::*;

    pub struct SkWorkerApplication {
        pub flatpak_worker: FlatpakWorker,
        pub flatpak_task_sender: Sender<FlatpakTask>,
        pub flatpak_task_receiver: Receiver<FlatpakTask>,
        pub flatpak_message_receiver: Receiver<FlatpakMessage>,

        pub dbus_connection: RefCell<Option<Connection>>,

        pub thread_pool: RefCell<Option<ThreadPool>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorkerApplication {
        const NAME: &'static str = "SkWorkerApplication";
        type ParentType = adw::Application;
        type Type = super::SkWorkerApplication;

        fn new() -> Self {
            // Channel for sending tasks to the Flatpak worker
            let (flatpak_task_sender, flatpak_task_receiver) = unbounded();
            // Channel for receiving update messages from running Flatpak tasks
            let (flatpak_message_sender, flatpak_message_receiver) = unbounded();
            let flatpak_worker = FlatpakWorker::new(flatpak_message_sender);

            let dbus_connection = RefCell::default();
            let thread_pool = RefCell::default();

            Self {
                flatpak_worker,
                flatpak_task_sender,
                flatpak_task_receiver,
                flatpak_message_receiver,
                dbus_connection,
                thread_pool,
            }
        }
    }

    impl ObjectImpl for SkWorkerApplication {}

    impl GtkApplicationImpl for SkWorkerApplication {}

    impl AdwApplicationImpl for SkWorkerApplication {}

    impl ApplicationImpl for SkWorkerApplication {
        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);
            debug!("Application -> startup");

            let fut = clone!(@weak app => async move {
                if let Err(err) = app.start_dbus_server().await{
                    error!("Unable to start DBus server: {}", err.to_string());
                    app.quit();
                }

                let f1 = app.receive_tasks();
                let f2 = app.receive_messages();
                futures::join!(f1, f2);
            });
            spawn!(fut);
        }

        fn activate(&self, app: &Self::Type) {
            self.parent_activate(app);
            debug!("Application -> activate");

            // Start worker threads if needed
            if app.imp().thread_pool.borrow().is_none() {
                debug!("Start worker thread pool...");
                let thread_pool = ThreadPool::new_named(
                    "souk_worker".into(),
                    WORKER_THREADS,
                    WORKER_THREADS,
                    Duration::from_secs(0),
                );
                thread_pool.start_core_threads();

                *app.imp().thread_pool.borrow_mut() = Some(thread_pool);
            }
        }

        fn shutdown(&self, app: &Self::Type) {
            self.parent_shutdown(app);
            debug!("Application -> shutdown");

            if let Some(thread_pool) = app.imp().thread_pool.borrow_mut().take() {
                debug!("Stop worker thread pool...");
                thread_pool.shutdown_join();
            }
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

        // Wait 15 seconds before worker quits because of inactivity
        app.set_inactivity_timeout(15000);

        // Start mainloop
        app.run();

        debug!("Quit.");
    }

    async fn start_dbus_server(&self) -> zbus::Result<()> {
        debug!("Start DBus server...");

        let flatpak_task_sender = self.imp().flatpak_task_sender.clone();
        let worker = WorkerServer {
            flatpak_task_sender,
        };

        let con = ConnectionBuilder::session()?
            .name(config::WORKER_APP_ID)?
            .serve_at(DBUS_PATH, worker)?
            .build()
            .await?;

        *self.imp().dbus_connection.borrow_mut() = Some(con);
        Ok(())
    }

    async fn receive_tasks(&self) {
        let imp = self.imp();

        let mut flatpak_receiver = imp.flatpak_task_receiver.clone();
        let flatpak_tasks = async move {
            while let Some(task) = flatpak_receiver.next().await {
                self.start_task(task).await;
            }
        };

        // Insert other kinds of tasks here, eg. appstream stuff

        let receiver = vec![flatpak_tasks];
        join_all(receiver).await;

        debug!("Stopped receiving tasks.");
    }

    /// Receives status/update messages, and reports them back to the main
    /// binary as a DBus signal
    // TODO: Make this generic, and not Flatpak specific
    async fn receive_messages(&self) {
        let imp = self.imp();

        let signal_ctxt = {
            let con = self.imp().dbus_connection.borrow();
            let con = con.as_ref().unwrap();
            SignalContext::new(con, DBUS_PATH).unwrap()
        };

        let mut receiver = imp.flatpak_message_receiver.clone();
        while let Some(message) = receiver.next().await {
            match message {
                FlatpakMessage::Progress(progress) => {
                    // Emit `transaction_progress` signal via dbus
                    WorkerServer::transaction_progress(&signal_ctxt, progress)
                        .await
                        .unwrap()
                }
                FlatpakMessage::Error(error) => {
                    // Emit `transaction_error` signal via dbus
                    WorkerServer::transaction_error(&signal_ctxt, error)
                        .await
                        .unwrap()
                }
            }
        }

        debug!("Stopped receiving messages.");
    }

    // TODO: Make the task here generic, and not Flatpak specific
    async fn start_task(&self, task: FlatpakTask) {
        let imp = self.imp();
        let (sender, mut receiver) = unbounded();

        // Activate gio application to ensure that thread pool is started
        self.activate();

        debug!("Start task: {:#?}", task);
        self.hold();

        // Own scope for `await_holding_refcell_ref` lint
        {
            let thread_pool = imp.thread_pool.borrow();
            if let Some(thread_pool) = &*thread_pool {
                thread_pool.spawn(
                    clone!(@strong imp.flatpak_worker as worker, @strong task => async move {
                        worker.process_task(task);
                        sender.send(true).await.unwrap();
                    }),
                );
            } else {
                error!("Unable to start task, thread pool is not available.");
                return;
            }
        }

        receiver.next().await;
        self.release();
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
