// Souk - app.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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
use std::env::var;
use std::time::Duration;

use adw::subclass::prelude::*;
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};
use lazy_static::lazy_static;
use once_cell::unsync::OnceCell;
use rusty_pool::ThreadPool;
use zbus::{Connection, ConnectionBuilder, SignalContext};

use crate::shared::config;
use crate::shared::task::response::TaskResponse;
use crate::shared::task::Task;
use crate::worker::appstream::AppstreamWorker;
use crate::worker::dbus_server::WorkerServer;
use crate::worker::flatpak::FlatpakWorker;

/// Specifies how many tasks can be executed in parallel
const WORKER_THREADS: usize = 4;

lazy_static! {
    static ref NO_INACTIVITY_TIMEOUT: bool = var("NO_INACTIVITY_TIMEOUT").is_ok();
}

mod imp {
    use super::*;

    pub struct SkWorkerApplication {
        task_sender: Sender<Task>,
        task_receiver: Receiver<Task>,
        cancel_sender: Sender<Task>,
        cancel_receiver: Receiver<Task>,
        response_receiver: Receiver<TaskResponse>,

        flatpak_worker: FlatpakWorker,
        appstream_worker: AppstreamWorker,

        dbus_connection: RefCell<Option<Connection>>,
        thread_pool: RefCell<Option<ThreadPool>>,

        hold_guard: OnceCell<gio::ApplicationHoldGuard>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkWorkerApplication {
        const NAME: &'static str = "SkWorkerApplication";
        type ParentType = adw::Application;
        type Type = super::SkWorkerApplication;

        fn new() -> Self {
            let (task_sender, task_receiver) = unbounded();
            let (cancel_sender, cancel_receiver) = unbounded();
            let (response_sender, response_receiver) = unbounded();

            let flatpak_worker = FlatpakWorker::new(response_sender.clone());
            let appstream_worker = AppstreamWorker::new(response_sender);

            let dbus_connection = RefCell::default();
            let thread_pool = RefCell::default();
            let hold_guard = OnceCell::default();

            Self {
                task_sender,
                task_receiver,
                cancel_sender,
                cancel_receiver,
                response_receiver,
                flatpak_worker,
                appstream_worker,
                dbus_connection,
                thread_pool,
                hold_guard,
            }
        }
    }

    impl ObjectImpl for SkWorkerApplication {}

    impl GtkApplicationImpl for SkWorkerApplication {}

    impl AdwApplicationImpl for SkWorkerApplication {}

    impl ApplicationImpl for SkWorkerApplication {
        fn startup(&self) {
            self.parent_startup();
            debug!("Application -> startup");

            let fut = clone!(@weak self as this => async move {
                if let Err(err) = this.start_dbus_server().await{
                    error!("Unable to start DBus server: {}", err.to_string());
                    this.obj().quit();
                }

                let f1 = this.receive_tasks();
                let f2 = this.receive_cancel_requests();
                let f3 = this.receive_responses();
                futures::join!(f1, f2, f3);
            });
            spawn!(fut);

            self.parent_activate();
            debug!("Application -> activate");

            // Start worker threads if needed
            if self.thread_pool.borrow().is_none() {
                debug!("Start worker thread pool...");
                let thread_pool = ThreadPool::new_named(
                    "souk_worker".into(),
                    WORKER_THREADS,
                    WORKER_THREADS,
                    Duration::from_secs(0),
                );
                thread_pool.start_core_threads();

                *self.thread_pool.borrow_mut() = Some(thread_pool);
            }

            if *NO_INACTIVITY_TIMEOUT {
                self.hold_guard.set(self.obj().hold()).unwrap();
            }
        }

        fn shutdown(&self) {
            self.parent_shutdown();
            debug!("Application -> shutdown");

            if let Some(thread_pool) = self.thread_pool.borrow_mut().take() {
                debug!("Stop worker thread pool...");
                thread_pool.shutdown_join();
            }
        }
    }

    impl SkWorkerApplication {
        async fn start_dbus_server(&self) -> zbus::Result<()> {
            debug!("Start DBus server...");

            let task_sender = self.task_sender.clone();
            let cancel_sender = self.cancel_sender.clone();
            let worker = WorkerServer {
                task_sender,
                cancel_sender,
            };

            let con = ConnectionBuilder::session()?
                .name(config::WORKER_APP_ID)?
                .serve_at(config::DBUS_PATH, worker)?
                .build()
                .await?;

            *self.dbus_connection.borrow_mut() = Some(con);
            Ok(())
        }

        fn start_task(&self, task: Task) {
            // Activate gio application to ensure that thread pool is started
            self.activate();

            debug!("Start task: {:#?}", task);
            let _ = self.obj().hold();

            // Own scope for `await_holding_refcell_ref` lint
            {
                let thread_pool = self.thread_pool.borrow();
                if let Some(thread_pool) = &*thread_pool {
                    // Flatpak task
                    if let Some(task) = task.flatpak_task() {
                        thread_pool.spawn(
                            clone!(@strong self.flatpak_worker as worker, @strong task => async move {
                                worker.process_task(task);
                            }),
                        );
                    }

                    // Appstream task
                    if let Some(task) = task.appstream_task() {
                        thread_pool.spawn(
                            clone!(@strong self.appstream_worker as worker, @strong task => async move {
                                worker.process_task(task);
                            }),
                        );
                    }
                } else {
                    error!("Unable to start task, thread pool is not available.");
                }
            }
        }

        async fn cancel_task(&self, task: Task) {
            debug!("Cancel task: {:#?}", task);

            if !task.cancellable {
                warn!("Task {} is not cancellable.", task.uuid);
                return;
            }

            // Flatpak task
            if task.flatpak_task().is_some() {
                self.flatpak_worker.cancel_task(&task.uuid);
            }

            // Appstream task
            if task.appstream_task().is_some() {
                self.appstream_worker.cancel_task(&task.uuid);
            }
        }

        async fn receive_tasks(&self) {
            let mut task_receiver = self.task_receiver.clone();
            while let Some(task) = task_receiver.next().await {
                self.start_task(task);
            }

            debug!("Stopped receiving tasks.");
        }

        async fn receive_cancel_requests(&self) {
            let mut cancel_receiver = self.cancel_receiver.clone();
            while let Some(task) = cancel_receiver.next().await {
                self.cancel_task(task).await;
            }

            debug!("Stopped receiving cancel requests.");
        }

        async fn receive_responses(&self) {
            let signal_ctxt = {
                let con = self.dbus_connection.borrow();
                let con = con.as_ref().unwrap();
                SignalContext::new(con, config::DBUS_PATH).unwrap()
            };

            let mut receiver = self.response_receiver.clone();
            while let Some(response) = receiver.next().await {
                let json = serde_json::to_string(&response).expect("Unable to serialize response");
                WorkerServer::task_response(&signal_ctxt, &json)
                    .await
                    .unwrap()
            }

            debug!("Stopped receiving responses.");
        }
    }
}

glib::wrapper! {
    pub struct SkWorkerApplication(ObjectSubclass<imp::SkWorkerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SkWorkerApplication {
    pub fn run() -> glib::ExitCode {
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

        let app: Self = glib::Object::builder()
            .property("application-id", &Some(app_id))
            .property("flags", gio::ApplicationFlags::IS_SERVICE)
            .build();

        // Wait 15 seconds before worker quits because of inactivity
        app.set_inactivity_timeout(15000);

        // Start mainloop
        app.run()
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
