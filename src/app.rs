// Souk - app.rs
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

use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::{clone, ObjectExt, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, FileChooserAction, FileChooserNative};
use once_cell::sync::{Lazy, OnceCell};

use crate::config;
use crate::flatpak::sideload::SkSideloadType;
use crate::flatpak::SkWorker;
use crate::i18n::i18n;
use crate::ui::sideload::SkSideloadWindow;
use crate::ui::{about_dialog, SkApplicationWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkApplication {
        pub window: OnceCell<WeakRef<SkApplicationWindow>>,
        pub worker: SkWorker,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkApplication {
        const NAME: &'static str = "SkApplication";
        type ParentType = adw::Application;
        type Type = super::SkApplication;
    }

    impl ObjectImpl for SkApplication {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "worker",
                    "",
                    "",
                    SkWorker::static_type(),
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "worker" => obj.worker().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl GtkApplicationImpl for SkApplication {}

    impl AdwApplicationImpl for SkApplication {}

    impl ApplicationImpl for SkApplication {
        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);

            debug!("Application -> startup");
            let app = app.downcast_ref::<super::SkApplication>().unwrap();

            // Setup `app` level GActions
            app.setup_gactions();
        }

        fn activate(&self, app: &Self::Type) {
            self.parent_activate(app);

            debug!("Application -> activate");
            let app = app.downcast_ref::<super::SkApplication>().unwrap();

            // If the window already exists, present it instead creating a new one again.
            if let Some(weak_window) = self.window.get() {
                weak_window.upgrade().unwrap().present();
                info!("Application window presented.");
                return;
            }

            // No window available -> we have to create one
            let window = app.create_window();
            let _ = self.window.set(window.downgrade());
            info!("Created application window.");
        }

        fn open(&self, app: &Self::Type, files: &[gio::File], hint: &str) {
            self.parent_open(app, files, hint);

            debug!("Application -> open");
            let app = app.downcast_ref::<super::SkApplication>().unwrap();

            for file in files {
                let sideload_type = SkSideloadType::determine_type(file);

                if sideload_type == SkSideloadType::Ref {
                    // TODO: Check if the FlatpakRef file is for a already added remote
                    let is_known_remote = false;
                    if is_known_remote {
                        app.activate();
                        // TODO: Open app details page for flatpak ref
                        return;
                    }
                }

                let _ = app.create_sideload_window(file);
            }
        }
    }
}

glib::wrapper! {
    pub struct SkApplication(ObjectSubclass<imp::SkApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SkApplication {
    pub fn run() {
        info!(
            "{} ({}) ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG
        );
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        // Create new GObject and downcast it into SkApplication
        let app = glib::Object::new::<SkApplication>(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
            ("resource-base-path", &Some("/de/haeckerfelix/Souk/")),
        ])
        .unwrap();

        // Start running gtk::Application
        app.run();
    }

    pub fn worker(&self) -> SkWorker {
        self.imp().worker.clone()
    }

    pub fn app_window(&self) -> Option<SkApplicationWindow> {
        if let Some(window) = self.imp().window.get() {
            window.upgrade()
        } else {
            None
        }
    }

    fn create_window(&self) -> SkApplicationWindow {
        let window = SkApplicationWindow::new();
        self.add_window(&window);

        window.present();
        window
    }

    fn create_sideload_window(&self, file: &gio::File) -> SkSideloadWindow {
        let window = SkSideloadWindow::new(file);
        self.add_window(&window);

        window.present();
        window
    }

    fn setup_gactions(&self) {
        // app.quit
        action!(self, "quit", move |_, _| {
            let app = SkApplication::default();
            for window in app.windows() {
                window.close();
            }
        });
        self.set_accels_for_action("app.quit", &["<primary>q"]);

        // app.about
        action!(self, "about", move |_, _| {
            let app = SkApplication::default();
            if let Some(window) = app.app_window() {
                about_dialog::show_about_dialog(&window);
            }
        });

        // app.open
        action!(self, "open", move |_, _| {
            SkApplication::default().show_filechooser();
        });
        self.set_accels_for_action("app.open", &["<primary>o"]);

        // app.refresh-installations
        action!(self, "refresh-installations", move |_, _| {
            let installations = SkApplication::default().worker().installations();
            if let Err(err) = installations.refresh() {
                error!(
                    "Unable to refresh Flatpak installations: {}",
                    err.to_string()
                );
                // TODO: Expose this in UI
            }
        });
        self.set_accels_for_action("app.refresh-installations", &["<primary>r"]);
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

        dialog.connect_response(
            clone!(@strong dialog, @weak self as this => move |_, resp| {
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
            }),
        );

        dialog.show();
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
