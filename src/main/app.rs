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

use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::{clone, ObjectExt, ParamSpec, Properties};
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{gio, glib, FileChooserAction, FileChooserNative};
use once_cell::sync::OnceCell;

use crate::main::flatpak::sideload::SkSideloadType;
use crate::main::i18n::i18n;
use crate::main::ui::debug::SkDebugWindow;
use crate::main::ui::sideload::SkSideloadWindow;
use crate::main::ui::{about_window, SkApplicationWindow};
use crate::main::worker::SkWorker;
use crate::shared::config;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkApplication)]
    pub struct SkApplication {
        #[property(get)]
        pub worker: SkWorker,

        pub window: OnceCell<WeakRef<SkApplicationWindow>>,
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

            // app.quit
            action!(
                obj,
                "quit",
                clone!(@weak obj => move |_, _| {
                    for window in obj.windows() {
                        window.close();
                    }
                })
            );
            obj.set_accels_for_action("app.quit", &["<primary>q"]);

            // app.about
            action!(
                obj,
                "about",
                clone!(@weak self as this => move |_, _| {
                    if let Some(window) = this.app_window() {
                        about_window::show(&window);
                    }
                })
            );

            // app.open
            action!(
                obj,
                "open",
                clone!(@weak self as this => move |_, _| {
                    this.show_filechooser();
                })
            );
            obj.set_accels_for_action("app.open", &["<primary>o"]);

            // app.refresh-installations
            action!(
                obj,
                "refresh-installations",
                clone!(@weak obj => move |_, _| {
                    let installations = obj.worker().installations();
                    if let Err(err) = installations.refresh() {
                        error!(
                            "Unable to refresh Flatpak installations: {}",
                            err.to_string()
                        );
                        // TODO: Expose this in UI
                    }
                })
            );
            obj.set_accels_for_action("app.refresh-installations", &["<primary>r"]);

            // app.debug
            action!(obj, "debug", move |_, _| {
                SkDebugWindow::new().show();
            });
            obj.set_accels_for_action("app.debug", &["<primary>d"]);
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

            // Trigger refresh of all available Flatpak installations/remotes
            self.obj().activate_action("refresh-installations", None);
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);
            debug!("Application -> open");

            for file in files {
                let sideload_type = SkSideloadType::determine_type(file);

                if sideload_type == SkSideloadType::Ref {
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

            dialog.connect_response(clone!(@weak dialog, @weak self as this => move |_, resp| {
                if resp == gtk::ResponseType::Accept {
                    let mut files = Vec::new();
                    for pos in 0..dialog.files().n_items() {
                    let file = dialog.files()
                        .item(pos)
                        .unwrap()
                        .downcast::<gio::File>()
                        .unwrap();
                        files.push(file);
                    }

                    this.open(&files, "");
                }
            }));

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
            .property("application-id", &Some(config::APP_ID))
            .property("flags", &gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", &Some("/de/haeckerfelix/Souk/"))
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
