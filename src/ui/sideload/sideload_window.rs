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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::File;
use glib::{subclass, ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use libflatpak::prelude::*;
use libflatpak::BundleRef;
use once_cell::sync::{Lazy, OnceCell};

use crate::config;
use crate::flatpak::sideload::{SkBundle, SkSideloadType};
use crate::i18n::i18n;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/sideload_window.ui")]
    pub struct SkSideloadWindow {
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub sideload_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub start_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub error_status_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,

        pub file: OnceCell<File>,
        pub type_: OnceCell<SkSideloadType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadWindow {
        const NAME: &'static str = "SkSideloadWindow";
        type ParentType = adw::Window;
        type Type = super::SkSideloadWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
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
                    ParamSpecEnum::new(
                        "type",
                        "Type",
                        "Type",
                        SkSideloadType::static_type(),
                        SkSideloadType::None as i32,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "type" => obj.type_().to_value(),
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
                "type" => self.type_.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_widgets();
            obj.setup_signals();
            obj.setup_gactions();
            obj.handle_file();
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
    pub fn new(file: &File, type_: &SkSideloadType) -> Self {
        glib::Object::new::<Self>(&[("file", file), ("type", type_)]).unwrap()
    }

    pub fn file(&self) -> File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn type_(&self) -> SkSideloadType {
        *self.imp().type_.get().unwrap()
    }

    fn setup_widgets(&self) {
        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            self.add_css_class("devel");
        }
    }

    fn setup_signals(&self) {}

    fn setup_gactions(&self) {}

    fn handle_file(&self) {
        let imp = self.imp();

        let package_button_text = i18n("Install");
        let repo_button_text = i18n("Add");

        let package_title_text = i18n("Install App");
        let repo_title_text = i18n("Add Software Source");

        // Adjust window for different sideload types
        match self.type_() {
            SkSideloadType::Bundle | SkSideloadType::Ref => {
                imp.start_button.set_label(&package_button_text);
                imp.window_title.set_title(&package_title_text);
            }
            SkSideloadType::Repo => {
                imp.start_button.set_label(&repo_button_text);
                imp.window_title.set_title(&repo_title_text);
            }
            _ => {
                let msg = i18n("Unknown or unsupported file format.");
                self.show_error_message(&msg);
                return;
            }
        }

        match self.type_() {
            SkSideloadType::Bundle => {
                let bundle = BundleRef::new(&self.file()).unwrap();
                let bundle = SkBundle::new(&bundle);

                self.imp()
                    .name_label
                    .set_text(&bundle.ref_().name().unwrap());
            }
            SkSideloadType::Ref => {}
            SkSideloadType::Repo => {}
            _ => (),
        }
    }

    fn show_error_message(&self, message: &str) {
        let imp = self.imp();

        imp.start_button.set_visible(false);
        imp.cancel_button.set_visible(false);

        imp.sideload_stack.set_visible_child_name("error");
        imp.error_status_page.set_description(Some(message));
    }
}
