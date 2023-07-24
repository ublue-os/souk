// Souk - debug_window.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::subclass;
use gtk::{glib, CompositeTemplate};

use crate::main::app::SkApplication;
use crate::main::flatpak::installation::{SkInstallation, SkRemote};
use crate::main::flatpak::package::SkPackage;
use crate::main::task::{SkOperation, SkTask};
use crate::main::ui::task::SkTaskProgressBar;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/debug_window.ui")]
    pub struct SkDebugWindow {
        #[template_child]
        current_tasks_columnview: TemplateChild<gtk::ColumnView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDebugWindow {
        const NAME: &'static str = "SkDebugWindow";
        type ParentType = adw::Window;
        type Type = super::SkDebugWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkDebugWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let worker = SkApplication::default().worker();

            let tree_model = gtk::TreeListModel::new(worker.tasks(), false, true, |item| {
                item.downcast_ref::<SkTask>()
                    .map(|task| task.operations().upcast())
            });

            let model = gtk::NoSelection::new(Some(tree_model));
            self.current_tasks_columnview.set_model(Some(&model));

            // Setup table columns
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let text = gtk::Inscription::new(None);
                text.set_hexpand(true);

                let expander = gtk::TreeExpander::new();
                expander.set_child(Some(&text));
                expander.set_width_request(125);

                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                item.set_child(Some(&expander));
            });

            factory.connect_bind(move |_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let expander = item
                    .child()
                    .unwrap()
                    .downcast::<gtk::TreeExpander>()
                    .unwrap();

                let listrow = item.item().unwrap().downcast::<gtk::TreeListRow>().unwrap();
                expander.set_list_row(Some(&listrow));

                let text = expander
                    .child()
                    .unwrap()
                    .downcast::<gtk::Inscription>()
                    .unwrap();

                if let Some(task) = listrow.item().unwrap().downcast_ref::<SkTask>() {
                    // Shorten uuid to first block for better readability
                    let mut uuid = task.uuid();
                    uuid.truncate(8);

                    text.set_text(Some(&uuid));
                } else if let Some(operation) =
                    listrow.item().unwrap().downcast_ref::<SkOperation>()
                {
                    text.set_text(Some(&format!("Operation {}", &operation.index())));
                }
            });
            self.add_column("Task", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);

                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkTask>("kind")
                    .bind(&text, "text", None::<&SkTask>);

                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("kind")
                    .bind(&text, "text", None::<&SkOperation>);
            });
            self.add_column("Kind", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);

                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("status")
                    .bind(&text, "text", None::<&SkTask>);

                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkTask>("status")
                    .bind(&text, "text", None::<&SkTask>);
            });
            self.add_column("Status", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let progressbar = SkTaskProgressBar::new();

                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                item.set_child(Some(&progressbar));
                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .bind(&progressbar, "task", None::<&gtk::TreeListRow>);
            });
            self.add_column("Progress", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);
                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("package")
                    .chain_property::<SkPackage>("name")
                    .bind(&text, "text", None::<&SkPackage>);
            });
            self.add_column("Ref", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);
                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("package")
                    .chain_property::<SkPackage>("branch")
                    .bind(&text, "text", None::<&SkPackage>);
            });
            self.add_column("Branch", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);
                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("package")
                    .chain_property::<SkPackage>("remote")
                    .chain_property::<SkRemote>("name")
                    .bind(&text, "text", None::<&SkRemote>);
            });
            self.add_column("Remote", factory);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_factory, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let text = Self::setup_text_widget(item);
                item.property_expression("item")
                    .chain_property::<gtk::TreeListRow>("item")
                    .chain_property::<SkOperation>("package")
                    .chain_property::<SkPackage>("remote")
                    .chain_property::<SkRemote>("installation")
                    .chain_property::<SkInstallation>("name")
                    .bind(&text, "text", None::<&SkInstallation>);
            });
            self.add_column("Installation", factory);
        }
    }

    impl WidgetImpl for SkDebugWindow {}

    impl WindowImpl for SkDebugWindow {}

    impl AdwWindowImpl for SkDebugWindow {}

    impl SkDebugWindow {
        fn add_column(&self, name: &str, factory: gtk::SignalListItemFactory) {
            let column = gtk::ColumnViewColumn::new(Some(name), Some(factory));
            column.set_resizable(true);
            self.current_tasks_columnview.append_column(&column);
        }

        fn setup_text_widget(item: &gtk::ListItem) -> gtk::Inscription {
            let text = gtk::Inscription::new(None);
            text.set_width_request(100);

            item.set_child(Some(&text));
            text
        }
    }
}

glib::wrapper! {
    pub struct SkDebugWindow(
        ObjectSubclass<imp::SkDebugWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

#[gtk::template_callbacks]
impl SkDebugWindow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for SkDebugWindow {
    fn default() -> Self {
        Self::new()
    }
}
