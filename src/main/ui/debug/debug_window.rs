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
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::main::app::SkApplication;
use crate::main::task::{SkTask, SkTaskModel, SkTaskStep};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/debug_window.ui")]
    pub struct SkDebugWindow {
        #[template_child]
        pub current_tasks_columnview: TemplateChild<gtk::ColumnView>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
        }
    }

    impl WidgetImpl for SkDebugWindow {}

    impl WindowImpl for SkDebugWindow {}

    impl AdwWindowImpl for SkDebugWindow {}
}

glib::wrapper! {
    pub struct SkDebugWindow(
        ObjectSubclass<imp::SkDebugWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

#[gtk::template_callbacks]
impl SkDebugWindow {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let worker = SkApplication::default().worker();

        // Combining the two tasks models into a single one
        let tasks = gio::ListStore::new(SkTaskModel::static_type());
        tasks.append(&worker.tasks_active());
        tasks.append(&worker.tasks_completed());
        let tasks = gtk::FlattenListModel::new(Some(&tasks));

        let tree_model = gtk::TreeListModel::new(&tasks, false, true, |item| {
            if item.type_().to_string() != "SkTask" {
                return None;
            }

            let task: &SkTask = item.downcast_ref().unwrap();
            Some(task.steps().upcast())
        });

        let model = gtk::NoSelection::new(Some(&tree_model));
        imp.current_tasks_columnview.set_model(Some(&model));

        // "Task" column

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            // TODO: Use gtk inscription
            let text = gtk::Label::new(None);
            let expander = gtk::TreeExpander::new();
            expander.set_child(Some(&text));

            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            item.set_child(Some(&expander));
        });

        factory.connect_bind(move |_factory, item| {
            let listrow = item.item().unwrap().downcast::<gtk::TreeListRow>().unwrap();

            let expander = item
                .child()
                .unwrap()
                .downcast::<gtk::TreeExpander>()
                .unwrap();

            expander.set_list_row(Some(&listrow));
            let text = expander.child().unwrap().downcast::<gtk::Label>().unwrap();

            if let Ok(task) = listrow.item().unwrap().downcast::<SkTask>() {
                text.set_text(&task.uuid());
            }

            if let Ok(step) = listrow.item().unwrap().downcast::<SkTaskStep>() {
                text.set_text(&format!("Step {}", step.index()));
            }
        });

        let column = gtk::ColumnViewColumn::new(Some("Task"), Some(&factory));
        imp.current_tasks_columnview.insert_column(0, &column);
    }
}

impl Default for SkDebugWindow {
    fn default() -> Self {
        Self::new()
    }
}
