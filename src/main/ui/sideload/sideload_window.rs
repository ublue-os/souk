// Souk - window.rs
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

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::{File, ListStore};
use glib::{clone, closure, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::{gio, glib, CompositeTemplate};
use once_cell::sync::{Lazy, OnceCell};

use super::SkRemoteRow;
use crate::main::app::SkApplication;
use crate::main::context::SkContext;
use crate::main::error::Error;
use crate::main::flatpak::dry_run::SkDryRun;
use crate::main::flatpak::installation::SkRemote;
use crate::main::flatpak::package::{SkPackage, SkPackageType};
use crate::main::flatpak::sideload::{SkSideloadType, SkSideloadable};
use crate::main::i18n::{i18n, i18n_f};
use crate::main::task::{SkTask, SkTaskStatus};
use crate::main::ui::badge::SkBadge;
use crate::main::ui::context::{SkContextBox, SkContextDetailRow};
use crate::main::ui::installation::SkInstallationListBox;
use crate::main::ui::task::SkTaskProgressBar;
use crate::main::ui::utils;
use crate::shared::{config, WorkerError};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/sideload_window.ui")]
    pub struct SkSideloadWindow {
        #[template_child]
        pub sideload_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub sideload_leaflet: TemplateChild<adw::Leaflet>,

        #[template_child]
        pub cancel_sideload_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub start_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub package_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub remotes_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub details_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub package_icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub package_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub package_developer_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub package_version_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub package_branch_badge: TemplateChild<SkBadge>,
        #[template_child]
        pub package_repository_badge: TemplateChild<SkBadge>,
        #[template_child]
        pub package_context_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub warn_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub no_updates_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub replacing_remote_row: TemplateChild<adw::ActionRow>,

        #[template_child]
        pub context_box: TemplateChild<SkContextBox>,

        #[template_child]
        pub installation_listbox: TemplateChild<SkInstallationListBox>,

        #[template_child]
        pub progress_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub progress_bar: TemplateChild<SkTaskProgressBar>,
        #[template_child]
        pub progress_status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub progress_download_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub done_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub done_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub launch_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub error_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub error_status_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub already_done_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub already_done_status_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub missing_runtime_status_page: TemplateChild<adw::StatusPage>,

        pub file: OnceCell<File>,
        pub sideloadable: RefCell<Option<SkSideloadable>>,

        pub task: OnceCell<SkTask>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadWindow {
        const NAME: &'static str = "SkSideloadWindow";
        type ParentType = adw::Window;
        type Type = super::SkSideloadWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
            klass.install_action("window.go-back", None, move |w, _, _| w.go_back());
            klass.install_action("window.select-installation", None, move |w, _, _| {
                w.go_back()
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkSideloadWindow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "file",
                        "",
                        "",
                        File::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecObject::new(
                        "sideloadable",
                        "",
                        "",
                        SkSideloadable::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "task",
                        "",
                        "",
                        SkTask::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.obj().file().to_value(),
                "sideloadable" => self.obj().sideloadable().to_value(),
                "task" => self.obj().task().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "file" => self.file.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let worker = SkApplication::default().worker();
            let preferred = worker.installations().preferred();
            self.installation_listbox
                .set_selected_installation(&preferred);

            self.obj().setup_widgets();
            self.obj().setup_actions();

            let fut = clone!(@weak self as this => async move {
                this.obj().update_sideloadable().await;
            });
            spawn!(fut);
        }
    }

    impl WidgetImpl for SkSideloadWindow {}

    impl WindowImpl for SkSideloadWindow {}

    impl AdwWindowImpl for SkSideloadWindow {}
}

glib::wrapper! {
    pub struct SkSideloadWindow(
        ObjectSubclass<imp::SkSideloadWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gio::ActionMap, gio::ActionGroup;
}

#[gtk::template_callbacks]
impl SkSideloadWindow {
    pub fn new(file: &File) -> Self {
        glib::Object::new::<Self>(&[("file", file)])
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn sideloadable(&self) -> Option<SkSideloadable> {
        self.imp().sideloadable.borrow().clone()
    }

    pub fn task(&self) -> Option<SkTask> {
        self.imp().task.get().cloned()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" {
            self.add_css_class("devel");
        }

        // Details View
        self.property_expression("sideloadable")
            .chain_property::<SkSideloadable>("package-dry-run")
            .chain_property::<SkDryRun>("package")
            .chain_property::<SkPackage>("branch")
            .bind(&imp.package_branch_badge.get(), "value", None::<&SkPackage>);

        self.property_expression("sideloadable")
            .chain_property::<SkSideloadable>("package-dry-run")
            .chain_property::<SkDryRun>("package")
            .chain_property::<SkPackage>("remote")
            .chain_property::<SkRemote>("name")
            .bind(
                &imp.package_repository_badge.get(),
                "value",
                None::<&SkPackage>,
            );

        // Progress View
        self.property_expression("task")
            .bind(&imp.progress_bar.get(), "task", None::<&SkTask>);

        self.property_expression("task")
            .chain_property::<SkTask>("status")
            .chain_closure::<String>(closure!(|_: Option<glib::Object>, status: SkTaskStatus| {
                status.to_string()
            }))
            .bind(&imp.progress_status_label.get(), "label", None::<&SkTask>);

        self.property_expression("task")
            .chain_property::<SkTask>("download-rate")
            .chain_closure::<String>(closure!(|_: Option<glib::Object>, download_rate: u64| {
                if download_rate != 0 {
                    i18n_f(
                        "Downloading data {}/s",
                        &[&glib::format_size(download_rate)],
                    )
                } else {
                    String::new()
                }
            }))
            .bind(&imp.progress_download_label.get(), "label", None::<&SkTask>);

        // When the installation changes, we also have to update the sideloadable,
        // since it depends on the installation
        imp.installation_listbox.connect_local(
            "notify::selected-installation",
            false,
            clone!(@weak self as this => @default-return None, move |_|{
                let fut = async move {
                    this.update_sideloadable().await;
                };
                spawn!(fut);
                None
            }),
        );
    }

    fn setup_actions(&self) {
        let actions = gio::SimpleActionGroup::new();
        self.insert_action_group("sideload", Some(&actions));

        // sideload.select-installation
        let action = gio::SimpleAction::new("select-installation", None);
        action.connect_activate(clone!(@weak self as this => move |_, _| {
            let imp = this.imp();

            let current = imp.sideload_leaflet.visible_child_name().unwrap().to_string();
            let select_installation = imp.sideload_leaflet.child_by_name("select-installation").unwrap();

            let page = match current.as_str() {
                "details" => Some(imp.sideload_leaflet.child_by_name("details").unwrap()),
                "missing-runtime" => Some(imp.sideload_leaflet.child_by_name("missing-runtime").unwrap()),
                "already-done" => Some(imp.sideload_leaflet.child_by_name("already-done").unwrap()),
                _ => None,
            };

            // Reorder select-installation to the correct position, so that "navigate-back" works properly
            if let Some(page) = page {
                imp.sideload_leaflet.reorder_child_after(&select_installation, Some(&page));
            }

            imp.sideload_leaflet.set_visible_child_name("select-installation");
        }));
        actions.add_action(&action);
    }

    /// Load the sideloadable for the selected file asynchronously
    async fn update_sideloadable(&self) {
        let imp = self.imp();
        imp.sideload_stack.set_visible_child_name("loading");

        let installation = imp.installation_listbox.selected_installation().unwrap();
        let file = self.file();

        let worker = SkApplication::default().worker();
        let sideloadable = worker.load_sideloadable(&file, &installation).await;

        match sideloadable {
            Ok(sideloadable) => {
                *imp.sideloadable.borrow_mut() = Some(sideloadable);
                self.notify("sideloadable");

                self.update_widgets();
                imp.sideload_stack.set_visible_child_name("leaflet");
            }
            Err(err) => {
                if let Error::Worker(worker_error) = &err {
                    match worker_error {
                        WorkerError::DryRunRuntimeNotFound(runtime) => {
                            self.show_missing_runtime_message(runtime)
                        }
                        _ => self.show_error_message(&err.message()),
                    }
                }

                match err {
                    Error::Worker(_) => (),
                    Error::UnsupportedSideloadType => {
                        let message = i18n("Unknown or unsupported file format.");
                        self.show_error_message(&message);
                    }
                    Error::GLib(err) => self.show_error_message(err.message()),
                    Error::ZBus(err) => {
                        let message = i18n_f(
                            "Unable to communicate with worker process: {}",
                            &[&err.to_string()],
                        );
                        self.show_error_message(&message);
                    }
                }
            }
        }
    }

    // TODO: Refactor this mess...
    fn update_widgets(&self) {
        let imp = self.imp();
        let sideloadable = self.sideloadable().unwrap();

        let app_start_button = i18n("Install");
        let update_start_button = i18n("Update");
        let repo_start_button = i18n("Add");

        let app_details_title = i18n("Install Package");
        let update_details_title = i18n("Update Package");
        let repo_details_title = i18n("Add Software Source");

        let app_progress_title = i18n("Installing Package");
        let update_progress_title = i18n("Updating Package");
        let repo_progress_title = i18n("Adding Software Source");

        let app_already_done_title = i18n("Already Installed");
        let repo_already_done_title = i18n("Already Added Source");
        let app_already_done_descr = i18n("This application or runtime is already installed.");
        let repo_already_done_descr = i18n("This software source is already added.");

        let app_done_title = i18n("Installation Complete");
        let update_done_title = i18n("Update Complete");
        let repo_done_title = i18n("Added Software Source");
        let app_done_descr = i18n("Successfully installed!");
        let update_done_descr = i18n("Successfully updated!");
        let repo_done_descr = i18n("Successfully added!");

        let app_error_title = i18n("Installation Failed");
        let update_error_title = i18n("Update Failed");
        let repo_error_title = i18n("Adding Source Failed");

        // Package
        let has_package = sideloadable.package_dry_run().is_some();
        imp.package_box.set_visible(has_package);

        if let Some(dry_run) = sideloadable.package_dry_run() {
            let package = dry_run.package();

            if dry_run.is_update() {
                imp.start_button.set_label(&update_start_button);
                imp.details_title.set_title(&update_details_title);
                imp.progress_title.set_title(&update_progress_title);
                imp.done_title.set_title(&update_done_title);
                imp.done_status_page
                    .set_description(Some(&update_done_descr));
                imp.error_title.set_title(&update_error_title);
            } else {
                imp.start_button.set_label(&app_start_button);
                imp.details_title.set_title(&app_details_title);
                imp.progress_title.set_title(&app_progress_title);
                imp.done_status_page.set_description(Some(&app_done_descr));
                imp.done_title.set_title(&app_done_title);
                imp.error_title.set_title(&app_error_title);
            }
            imp.already_done_title.set_title(&app_already_done_title);
            imp.already_done_status_page
                .set_description(Some(&app_already_done_descr));

            // Only display launch button if it's an app
            if package.type_() == SkPackageType::App {
                imp.launch_button.set_visible(true);
            }

            // Show warn message for packages without update source
            let uninstall_before_install_source = !dry_run.has_update_source();
            imp.no_updates_row
                .set_visible(uninstall_before_install_source);

            // Show warn message when package is already installed, but from a different
            // remote
            if let Some(remote) = dry_run.is_replacing_remote().as_ref() {
                imp.replacing_remote_row.set_visible(true);
                let msg = i18n_f("This package is already installed from \"{}\", during the installation the old version will be uninstalled first", &[&remote.name()]);
                imp.replacing_remote_row.set_subtitle(&msg);
            } else {
                imp.replacing_remote_row.set_visible(false);
            }

            imp.warn_group.set_visible(
                !(!imp.no_updates_row.is_visible() && !imp.replacing_remote_row.is_visible()),
            );

            // Context information
            let contexts = ListStore::new(SkContext::static_type());

            let download_size_context = dry_run.download_size_context();
            contexts.append(&download_size_context);

            let installed_size_context = dry_run.installed_size_context();
            contexts.append(&installed_size_context);

            let permissions_context = dry_run.permissions_context();
            contexts.append(&permissions_context);

            imp.package_context_listbox.bind_model(
                Some(&contexts),
                clone!(@weak self as this => @default-panic, move |context|{
                    let context: &SkContext = context.downcast_ref().unwrap();
                    let row = SkContextDetailRow::new(&context.summary(), true);
                    row.set_activatable(true);
                    row.set_subtitle_lines(2);

                    row.connect_activated(clone!(@weak this, @weak context => move |_|{
                        this.imp().context_box.set_context(&context);
                        this.imp().sideload_leaflet.set_visible_child_name("context-information");
                    }));

                    row.upcast()
                }),
            );

            // Appstream information
            let asi = dry_run.appstream();
            imp.package_name_label.set_text(&asi.name());
            imp.package_icon_image.set_paintable(Some(&asi.icon()));
            imp.package_developer_label.set_text(&asi.developer_name());
            imp.package_version_label.set_text(&asi.version_text());

            // We don't support updating .flatpakrefs through sideloading, since the
            // installation would fail with "x is already installed". Only bundles can be
            // updated.
            if dry_run.is_update() && sideloadable.type_() == SkSideloadType::Ref {
                imp.sideload_leaflet.set_visible_child_name("already-done");
                return;
            }
        }

        // Remotes / Repositories
        imp.remotes_box
            .set_visible(!sideloadable.remotes().snapshot().is_empty());
        utils::clear_box(&imp.remotes_box);

        if !sideloadable.remotes().snapshot().is_empty() {
            let group = adw::PreferencesGroup::new();
            imp.remotes_box.append(&group);

            for object in sideloadable.remotes().snapshot() {
                let remote: SkRemote = object.downcast().unwrap();
                let remote_row = SkRemoteRow::new(&remote);
                group.add(&remote_row);
            }

            if sideloadable.package_dry_run().is_none() {
                let remotes = sideloadable.remotes();
                let remote: SkRemote = remotes.item(0).unwrap().downcast().unwrap();
                let name = if !remote.title().is_empty() {
                    remote.title()
                } else {
                    i18n("Software Source")
                };

                // The other widgets are bound to the package name / icon
                imp.package_name_label.set_text(&name);
                imp.package_icon_image.set_icon_name(Some("gear-symbolic"));

                imp.start_button.set_label(&repo_start_button);
                imp.details_title.set_title(&repo_details_title);
                imp.progress_title.set_title(&repo_progress_title);
                imp.done_title.set_title(&repo_done_title);
                imp.already_done_title.set_title(&repo_already_done_title);
                imp.error_title.set_title(&repo_error_title);
                imp.done_status_page.set_description(Some(&repo_done_descr));
                imp.already_done_status_page
                    .set_description(Some(&repo_already_done_descr));

                self.set_default_height(430);
                let msg = i18n("New applications can be obtained from this source. Only proceed if you trust this source.");
                group.set_description(Some(&msg));
            } else {
                let msg = i18n("This package adds a new software source. New applications can be obtained from it. Only proceed with the installation if you trust this source.");
                group.set_description(Some(&msg));
            };
        }

        // Nothing changes
        if sideloadable.no_changes() {
            imp.sideload_leaflet.set_visible_child_name("already-done");
        } else {
            imp.sideload_leaflet.set_visible_child_name("details");
        }
    }

    fn go_back(&self) {
        self.imp()
            .sideload_leaflet
            .navigate(adw::NavigationDirection::Back);
    }

    #[template_callback]
    fn start_sideload_clicked(&self) {
        let fut = clone!(@weak self as this => async move{
            this.start_sideload().await;
        });
        spawn!(fut);
    }

    async fn start_sideload(&self) {
        let imp = self.imp();
        let worker = SkApplication::default().worker();
        let sideloadable = self.sideloadable().unwrap();

        match sideloadable.type_() {
            SkSideloadType::Bundle | SkSideloadType::Ref => {
                let task = match sideloadable.sideload(&worker).await {
                    Ok(task) => task.unwrap(),
                    Err(err) => {
                        self.show_error_message(&err.to_string());
                        return;
                    }
                };

                self.handle_sideload_task(&task);
                imp.task.set(task).unwrap();
                self.notify("task");
            }
            SkSideloadType::Repo => {
                match sideloadable.sideload(&worker).await {
                    Ok(_) => imp.sideload_leaflet.set_visible_child_name("done"),
                    Err(err) => self.show_error_message(&err.message()),
                };
            }
            _ => (),
        }
    }

    fn handle_sideload_task(&self, task: &SkTask) {
        let imp = self.imp();
        imp.sideload_leaflet.set_visible_child_name("progress");

        task.connect_local(
            "done",
            false,
            clone!(@weak self as this => @default-return None, move |_|{
                this.imp().sideload_leaflet.set_visible_child_name("done");
                None
            }),
        );

        task.connect_local(
            "cancelled",
            false,
            clone!(@weak self as this => @default-return None, move |_|{
                this.close();
                None
            }),
        );

        task.connect_local(
            "error",
            false,
            clone!(@weak self as this => @default-return None, move |error|{
                let msg: String = error[1].get().unwrap();
                this.show_error_message(&msg);
                None
            }),
        );
    }

    #[template_callback]
    fn cancel_sideload_clicked(&self) {
        let fut = clone!(@weak self as this => async move{
            let imp = this.imp();
            let task = imp.task.get().unwrap();
            let worker = SkApplication::default().worker();

            imp.cancel_sideload_button.set_sensitive(false);
            if let Err(err) = worker.cancel_task(task).await {
                this.show_error_message(&err.to_string());
            }
        });
        spawn!(fut);
    }

    #[template_callback]
    fn launch_app_clicked(&self) {
        let fut = clone!(@weak self as this => async move{
            let sideloadable = this.sideloadable().unwrap();
            let installation = sideloadable.installation();

            let package = sideloadable.package_dry_run().unwrap().package();
            installation.launch_app(&package);

            this.close();
        });
        spawn!(fut);
    }

    fn show_error_message(&self, message: &str) {
        let imp = self.imp();

        imp.sideload_leaflet.set_visible_child_name("error");
        imp.sideload_stack.set_visible_child_name("leaflet");

        imp.error_status_page.set_description(Some(message));
    }

    fn show_missing_runtime_message(&self, runtime: &str) {
        let imp = self.imp();

        imp.sideload_leaflet
            .set_visible_child_name("missing-runtime");
        imp.sideload_stack.set_visible_child_name("leaflet");

        let message = i18n_f(
            "The required runtime <tt>{}</tt> could not be found. Possibly the runtime is available in a different installation.",
            &[runtime],
        );
        imp.missing_runtime_status_page
            .set_description(Some(&message));
    }
}
