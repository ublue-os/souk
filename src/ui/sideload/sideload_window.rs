// Souk - window.rs
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

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::File;
use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use libflatpak::prelude::*;
use libflatpak::RefKind;
use once_cell::sync::{Lazy, OnceCell};

use crate::app::SkApplication;
use crate::config;
use crate::error::Error;
use crate::flatpak::sideload::SkSideloadable;
use crate::flatpak::SkTransaction;
use crate::i18n::{i18n, i18n_f};
use crate::ui::SkInstallationListBox;

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
        pub details_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub package_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub package_download_size_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub package_installed_size_row: TemplateChild<adw::ActionRow>,

        #[template_child]
        pub installation_listbox: TemplateChild<SkInstallationListBox>,

        #[template_child]
        pub progress_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub progress_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub done_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub launch_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub error_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub error_status_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub already_done_title: TemplateChild<adw::WindowTitle>,

        #[template_child]
        pub missing_runtime_status_page: TemplateChild<adw::StatusPage>,

        pub file: OnceCell<File>,
        pub sideloadable: RefCell<Option<SkSideloadable>>,

        pub transaction: OnceCell<SkTransaction>,
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
                        "File",
                        "File",
                        File::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecObject::new(
                        "sideloadable",
                        "Sideloadable",
                        "Sideloadable",
                        SkSideloadable::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "sideloadable" => obj.sideloadable().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "file" => self.file.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let worker = SkApplication::default().worker();
            let preferred = worker.preferred_installation();
            self.installation_listbox.set_installation(&preferred);

            obj.setup_widgets();
            obj.setup_actions();

            let fut = clone!(@weak obj => async move {
                obj.update_sideloadable().await;
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
        glib::Object::new::<Self>(&[("file", file)]).unwrap()
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn sideloadable(&self) -> Option<SkSideloadable> {
        self.imp().sideloadable.borrow().clone()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            self.add_css_class("devel");
        }

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

    async fn update_sideloadable(&self) {
        let imp = self.imp();
        imp.sideload_stack.set_visible_child_name("loading");

        let installation = imp.installation_listbox.selected_installation().unwrap();
        let installation_uuid = installation.uuid();
        let file = self.file();

        let worker = SkApplication::default().worker();
        let sideloadable = worker.load_sideloadable(&file, &installation_uuid).await;

        match sideloadable {
            Ok(sideloadable) => {
                *imp.sideloadable.borrow_mut() = Some(sideloadable);
                self.notify("sideloadable");

                self.update_widgets();
                imp.sideload_stack.set_visible_child_name("leaflet");
            }
            Err(err) => match err {
                Error::DryRunRuntimeNotFound(runtime) => {
                    self.show_missing_runtime_message(&runtime)
                }
                Error::DryRunError(message) => self.show_error_message(&message),
                Error::UnsupportedSideloadType => {
                    let message = i18n("Unknown or unsupported file format.");
                    self.show_error_message(&message);
                }
                Error::DbusError(_) => {
                    let message = i18n("Unable to communicate with worker process.");
                    self.show_error_message(&message);
                }
            },
        }
    }

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

        let app_done_title = i18n("Installation Complete");
        let update_done_title = i18n("Update Complete");
        let repo_done_title = i18n("Added Software Source");

        let app_error_title = i18n("Installation Failed");
        let update_error_title = i18n("Update Failed");
        let repo_error_title = i18n("Adding Source Failed");

        // Setup window titles and headerbar buttons
        if sideloadable.contains_package() {
            if sideloadable.is_update() {
                imp.start_button.set_label(&update_start_button);
                imp.details_title.set_title(&update_details_title);
                imp.progress_title.set_title(&update_progress_title);
                imp.done_title.set_title(&update_done_title);
                imp.error_title.set_title(&update_error_title);
            } else {
                imp.start_button.set_label(&app_start_button);
                imp.details_title.set_title(&app_details_title);
                imp.progress_title.set_title(&app_progress_title);
                imp.done_title.set_title(&app_done_title);
                imp.error_title.set_title(&app_error_title);
            }
            imp.already_done_title.set_title(&app_already_done_title);
        }

        if sideloadable.contains_repository() && !sideloadable.contains_package() {
            imp.start_button.set_label(&repo_start_button);
            imp.details_title.set_title(&repo_details_title);
            imp.progress_title.set_title(&repo_progress_title);
            imp.done_title.set_title(&repo_done_title);
            imp.already_done_title.set_title(&repo_already_done_title);
            imp.error_title.set_title(&repo_error_title);
        }

        if sideloadable.is_already_done() {
            imp.sideload_leaflet.set_visible_child_name("already-done");
            return;
        } else {
            imp.sideload_leaflet.set_visible_child_name("details");
        }

        // Hide launch button if sideload content is not an app
        if sideloadable.ref_().kind() != RefKind::App {
            imp.launch_button.set_visible(false);
        }

        // Setup details page
        if sideloadable.contains_package() {
            imp.package_name_label
                .set_text(&sideloadable.ref_().format_ref().unwrap());

            let size = glib::format_size(sideloadable.download_size());
            let download_string = i18n_f("Up to {} download", &[&size]);
            imp.package_download_size_row.set_title(&download_string);

            let size = glib::format_size(sideloadable.installed_size());
            let installed_string = i18n_f("Up to {} installed size", &[&size]);
            imp.package_installed_size_row.set_title(&installed_string);
        }
    }

    fn go_back(&self) {
        self.imp()
            .sideload_leaflet
            .navigate(adw::NavigationDirection::Back);
    }

    #[template_callback]
    fn start_sideload(&self) {
        let fut = clone!(@weak self as this => async move{
            this.start_transaction().await;
        });
        spawn!(fut);
    }

    async fn start_transaction(&self) {
        let imp = self.imp();
        let sideloadable = self.sideloadable().unwrap();

        imp.sideload_leaflet.set_visible_child_name("progress");

        // Start sideloading the sideloadable, and track the transaction
        let transaction = match sideloadable.sideload().await {
            Ok(transaction) => transaction,
            Err(err) => {
                self.show_error_message(&err.to_string());
                return;
            }
        };

        transaction
            .bind_property("progress", &imp.progress_bar.get(), "fraction")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        transaction.connect_local(
            "notify::current-operation",
            false,
            clone!(@weak self as this, @weak transaction => @default-return None, move |_|{
                let imp = this.imp();

                let msg = format!(
                    "{} {} ({}/{})",
                    transaction.current_operation_type(),
                    transaction.current_operation_ref().unwrap().name().unwrap(),
                    transaction.current_operation(),
                    transaction.operations_count()
                );

                imp.progress_label.set_text(&msg);
                None
            }),
        );

        transaction.connect_local(
            "done",
            false,
            clone!(@weak self as this => @default-return None, move |_|{
                this.imp().sideload_leaflet.set_visible_child_name("done");
                None
            }),
        );

        transaction.connect_local(
            "cancelled",
            false,
            clone!(@weak self as this => @default-return None, move |_|{
                this.close();
                None
            }),
        );

        transaction.connect_local(
            "error",
            false,
            clone!(@weak self as this => @default-return None, move |error|{
                let msg: String = error[1].get().unwrap();
                this.show_error_message(&msg);
                None
            }),
        );

        imp.transaction.set(transaction).unwrap();
    }

    #[template_callback]
    fn cancel_sideload(&self) {
        let fut = clone!(@weak self as this => async move{
            let imp = this.imp();
            let uuid = imp.transaction.get().unwrap().uuid();
            let worker = SkApplication::default().worker();

            imp.cancel_sideload_button.set_sensitive(false);
            if let Err(err) = worker.cancel_transaction(&uuid).await {
                this.show_error_message(&err.to_string());
            }
        });
        spawn!(fut);
    }

    #[template_callback]
    fn launch_app(&self) {
        let fut = clone!(@weak self as this => async move{
            let worker = SkApplication::default().worker();
            let sideloadable = this.sideloadable().unwrap();
            let installation_uuid = sideloadable.installation_uuid();

            let _ = worker.launch_app(&installation_uuid, &sideloadable.ref_(), &sideloadable.commit()).await;
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
