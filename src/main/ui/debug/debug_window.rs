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
use glib::{closure, subclass};
use gtk::{glib, CompositeTemplate};

use crate::main::app::SkApplication;
use crate::main::flatpak::installation::{SkInstallation, SkRemote};
use crate::main::flatpak::package::SkPackage;
use crate::main::task::{SkTask, SkTaskModel};
use crate::main::ui::task::SkTaskProgressBar;

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
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_widgets();
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
        glib::Object::new()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let worker = SkApplication::default().worker();

        let tree_model = gtk::TreeListModel::new(worker.tasks(), false, true, |item| {
            let task: &SkTask = item.downcast_ref().unwrap();
            Some(task.dependencies().upcast())
        });

        let model = gtk::NoSelection::new(Some(tree_model));
        imp.current_tasks_columnview.set_model(Some(&model));

        // Setup table columns
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            let text = gtk::Label::new(None);
            let expander = gtk::TreeExpander::new();
            expander.set_child(Some(&text));

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

            // Hide expander icon if there are no dependency tasks
            item.property_expression("item")
                .chain_property::<gtk::TreeListRow>("item")
                .chain_property::<SkTask>("dependencies")
                .chain_closure::<bool>(closure!(
                    |_: Option<glib::Object>, deps: Option<SkTaskModel>| {
                        if let Some(deps) = deps {
                            deps.n_items() == 0
                        } else {
                            false
                        }
                    }
                ))
                .bind(&expander, "hide-expander", None::<&SkTask>);

            let listrow = item.item().unwrap().downcast::<gtk::TreeListRow>().unwrap();
            expander.set_list_row(Some(&listrow));

            let task = listrow.item().unwrap().downcast::<SkTask>().unwrap();

            let text = expander.child().unwrap().downcast::<gtk::Label>().unwrap();
            text.set_text(&task.uuid());
        });
        self.add_column("Task", factory);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let text = Self::setup_text_widget(item);
            item.property_expression("item")
                .chain_property::<gtk::TreeListRow>("item")
                .chain_property::<SkTask>("type")
                .bind(&text, "label", None::<&SkTask>);
        });
        self.add_column("Type", factory);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let text = Self::setup_text_widget(item);
            item.property_expression("item")
                .chain_property::<gtk::TreeListRow>("item")
                .chain_property::<SkTask>("status")
                .bind(&text, "label", None::<&SkTask>);
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
                .chain_property::<SkTask>("package")
                .chain_property::<SkPackage>("name")
                .bind(&text, "label", None::<&SkPackage>);
        });
        self.add_column("Ref", factory);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let text = Self::setup_text_widget(item);
            item.property_expression("item")
                .chain_property::<gtk::TreeListRow>("item")
                .chain_property::<SkTask>("package")
                .chain_property::<SkPackage>("remote")
                .chain_property::<SkRemote>("name")
                .bind(&text, "label", None::<&SkRemote>);
        });
        self.add_column("Remote", factory);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let text = Self::setup_text_widget(item);
            item.property_expression("item")
                .chain_property::<gtk::TreeListRow>("item")
                .chain_property::<SkTask>("package")
                .chain_property::<SkPackage>("remote")
                .chain_property::<SkRemote>("installation")
                .chain_property::<SkInstallation>("name")
                .bind(&text, "label", None::<&SkInstallation>);
        });
        self.add_column("Installation", factory);
    }

    fn add_column(&self, name: &str, factory: gtk::SignalListItemFactory) {
        let column = gtk::ColumnViewColumn::new(Some(name), Some(factory));
        self.imp().current_tasks_columnview.append_column(&column);
    }

    fn setup_text_widget(item: &gtk::ListItem) -> gtk::Label {
        // TODO: Use gtk inscription
        let text = gtk::Label::new(None);
        text.set_xalign(0.0);

        item.set_child(Some(&text));
        text
    }
}

impl Default for SkDebugWindow {
    fn default() -> Self {
        Self::new()
    }
}
