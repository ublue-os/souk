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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass};
use gtk::{gio, glib, CompositeTemplate};

use crate::main::app::SkApplication;
use crate::main::i18n::{i18n, i18n_f};
use crate::main::task::SkTask;
use crate::main::ui::main::SkSidebar;
use crate::main::ui::page::SkInstalledPage;
use crate::main::ui::SkProgressBar;
use crate::shared::config;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/window.ui")]
    pub struct SkApplicationWindow {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub initial_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub initial_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub initial_progressbar: TemplateChild<SkProgressBar>,

        #[template_child]
        pub sidebar: TemplateChild<SkSidebar>,
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        pub installed_page: TemplateChild<SkInstalledPage>,

        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkApplicationWindow {
        const NAME: &'static str = "SkApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::SkApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();

            // Add devel style class for development builds
            if config::PROFILE == "development" {
                self.obj().add_css_class("devel");
            }

            // DND support for sideloading
            let drop_target =
                gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
            drop_target.connect_drop(move |_, data, _, _| {
                if let Ok(filelist) = data.get::<gdk::FileList>() {
                    let app = SkApplication::default();
                    app.open(&filelist.files(), "");
                    return true;
                }
                false
            });
            self.status_page.add_controller(drop_target);

            // Initial view
            self.initial_status_page.set_icon_name(Some(config::APP_ID));
            let title = i18n_f("Welcome to {}", &[&config::NAME]);
            self.initial_status_page.set_title(&title);
        }
    }

    impl WidgetImpl for SkApplicationWindow {}

    impl WindowImpl for SkApplicationWindow {}

    impl ApplicationWindowImpl for SkApplicationWindow {}

    impl AdwApplicationWindowImpl for SkApplicationWindow {}
}

glib::wrapper! {
    pub struct SkApplicationWindow(
        ObjectSubclass<imp::SkApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

#[gtk::template_callbacks]
impl SkApplicationWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn show_initial_view(&self, task: &SkTask) {
        let imp = self.imp();
        imp.stack.set_visible_child(&imp.initial_view.get());

        task.bind_property("progress", &imp.initial_progressbar.get(), "fraction")
            .build();

        task.connect_local(
            "done",
            false,
            clone!(
                #[weak]
                imp,
                #[upgrade_or]
                None,
                move |_| {
                    imp.stack.set_visible_child(&imp.split_view.get());
                    None
                }
            ),
        );

        task.connect_local(
            "error",
            false,
            clone!(#[weak] imp , #[upgrade_or] None, move |_|{
                let msg = i18n("Unable to load software catalogue. Make sure you have a working internet connection and restart the application.");
                imp.initial_status_page.set_description(Some(&msg));
                imp.initial_progressbar.set_visible(false);
                None
            }),
        );
    }
}

impl Default for SkApplicationWindow {
    fn default() -> Self {
        Self::new()
    }
}
