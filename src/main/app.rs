// Souk - app.rs
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

use std::cell::OnceCell;

use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::{clone, ParamSpec, Properties};
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{gio, glib, FileChooserAction, FileChooserNative};

use crate::main::appstream::utils;
use crate::main::flatpak::sideload::SkSideloadKind;
use crate::main::i18n::i18n;
use crate::main::ui::about_window;
use crate::main::ui::debug::SkDebugWindow;
use crate::main::ui::main::SkApplicationWindow;
use crate::main::ui::sideload::SkSideloadWindow;
use crate::main::worker::SkWorker;
use crate::shared::config;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkApplication)]
    pub struct SkApplication {
        #[property(get)]
        worker: SkWorker,

        window: OnceCell<WeakRef<SkApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkApplication {
        const NAME: &'static str = "SkApplication";
        type ParentType = adw::Application;
        type Type = super::SkApplication;
    }

    impl ObjectImpl for SkApplication {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }
    }

    impl GtkApplicationImpl for SkApplication {}

    impl AdwApplicationImpl for SkApplication {}

    impl ApplicationImpl for SkApplication {
        fn startup(&self) {
            self.parent_startup();
            debug!("Application -> startup");

            // Setup `app` level GActions
            let obj = self.obj();

            let actions = [
                gio::ActionEntryBuilder::new("quit")
                    .activate(|app: &super::SkApplication, _, _| {
                        for window in app.windows() {
                            window.close();
                        }
                    })
                    .build(),
                gio::ActionEntryBuilder::new("about")
                    .activate(|app: &Self::Type, _, _| {
                        if let Some(window) = app.imp().app_window() {
                            about_window::show(&window);
                        }
                    })
                    .build(),
                gio::ActionEntryBuilder::new("open")
                    .activate(|app: &super::SkApplication, _, _| {
                        app.imp().show_filechooser();
                    })
                    .build(),
                gio::ActionEntryBuilder::new("refresh-installations")
                    .activate(|app: &super::SkApplication, _, _| {
                        let installations = app.worker().installations();
                        installations.refresh();
                    })
                    .build(),
                gio::ActionEntryBuilder::new("debug")
                    .activate(|_, _, _| {
                        SkDebugWindow::new().show();
                    })
                    .build(),
            ];
            obj.add_action_entries(actions);

            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.open", &["<primary>o"]);
            obj.set_accels_for_action("app.refresh-installations", &["<primary>r"]);
            obj.set_accels_for_action("app.debug", &["<primary>d"]);

            // Trigger refresh of all available Flatpak installations/remotes
            self.obj().activate_action("refresh-installations", None);
        }

        fn activate(&self) {
            self.parent_activate();
            debug!("Application -> activate");

            // If the window already exists, present it instead creating a new one again.
            if let Some(weak_window) = self.window.get() {
                weak_window.upgrade().unwrap().present();
                info!("Application window presented.");
                return;
            }

            // No window available -> we have to create one
            let window = self.create_window();
            let _ = self.window.set(window.downgrade());
            info!("Created application window.");

            let fut = clone!(
                #[weak(rename_to = this)]
                self,
                async move {
                    this.populate_views().await;
                }
            );
            crate::main::spawn_future_local(fut);
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);
            debug!("Application -> open");

            for file in files {
                let sideload_kind = SkSideloadKind::determine_kind(file);

                if sideload_kind == SkSideloadKind::Ref {
                    // TODO: Check if the FlatpakRef file is for a already added remote
                    let is_known_remote = false;
                    if is_known_remote {
                        self.obj().activate();
                        // TODO: Open app details page for flatpak ref
                        return;
                    }
                }

                let _ = self.create_sideload_window(file);
            }
        }
    }

    impl SkApplication {
        async fn populate_views(&self) {
            // Wait till worker is ready, otherwise there's a chance that responses get lost
            self.worker.wait_ready().await;

            // Show initial view if there's no appstream data available to display
            if !utils::check_appstream_silo_exists() {
                debug!("No appstream data available, trigger update.");
                let task = self
                    .worker
                    .update_appstream()
                    .await
                    .expect("Unable to spawn update appstream task");

                if let Some(window) = self.app_window() {
                    window.show_initial_view(&task);
                }
            } else {
                // Appstream data exists -> update it
                // TODO: this ^
            }
        }

        fn app_window(&self) -> Option<SkApplicationWindow> {
            if let Some(window) = self.window.get() {
                window.upgrade()
            } else {
                None
            }
        }

        fn create_window(&self) -> SkApplicationWindow {
            let window = SkApplicationWindow::new();
            self.obj().add_window(&window);

            window.present();
            window
        }

        fn create_sideload_window(&self, file: &gio::File) -> SkSideloadWindow {
            let window = SkSideloadWindow::new(file);
            self.obj().add_window(&window);

            window.present();
            window
        }

        fn show_filechooser(&self) {
            let window = self.app_window();

            let dialog = FileChooserNative::new(
                Some(&i18n("Open Flatpak package or repository")),
                window.as_ref(),
                FileChooserAction::Open,
                Some(&i18n("_Open")),
                None,
            );

            dialog.set_modal(true);
            dialog.set_select_multiple(true);

            // Set a filter to only show flatpak files
            let flatpak_filter = gtk::FileFilter::new();
            flatpak_filter.set_name(Some(&i18n("Flatpak Files")));
            flatpak_filter.add_mime_type("application/vnd.flatpak");
            flatpak_filter.add_mime_type("application/vnd.flatpak.repo");
            flatpak_filter.add_mime_type("application/vnd.flatpak.ref");
            dialog.add_filter(&flatpak_filter);

            // Set a filter to show all files
            let all_filter = gtk::FileFilter::new();
            all_filter.set_name(Some(&i18n("All Files")));
            all_filter.add_pattern("*");
            dialog.add_filter(&all_filter);

            dialog.connect_response(clone!(
                #[weak]
                dialog,
                #[weak(rename_to = this)]
                self,
                move |_, resp| {
                    if resp == gtk::ResponseType::Accept {
                        let mut files = Vec::new();
                        for pos in 0..dialog.files().n_items() {
                            let file = dialog
                                .files()
                                .item(pos)
                                .unwrap()
                                .downcast::<gio::File>()
                                .unwrap();
                            files.push(file);
                        }

                        this.open(&files, "");
                    }
                }
            ));

            dialog.show();
        }
    }
}

glib::wrapper! {
    pub struct SkApplication(ObjectSubclass<imp::SkApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SkApplication {
    pub fn run() -> glib::ExitCode {
        debug!(
            "{} ({}) ({}) - Version {} ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG,
            config::VERSION,
            config::PROFILE
        );

        // Create new GObject and downcast it into SkApplication
        let app: Self = glib::Object::builder()
            .property("application-id", Some(config::APP_ID))
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", Some("/de/haeckerfelix/Souk/"))
            .build();

        // Start running gtk::Application
        app.run()
    }
}

impl Default for SkApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
