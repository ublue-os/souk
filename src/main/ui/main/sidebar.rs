// Souk - sidebar.rs
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

use std::cell::{Cell, OnceCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, ParamSpec, Properties};
use gtk::{glib, CompositeTemplate};

use super::SkSidebarItemRow;

mod imp {
    use super::*;

    enum RowPosition {
        Top,
        Middle,
        Bottom,
    }

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::SkSidebar)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/sidebar.ui")]
    pub struct SkSidebar {
        #[property(get, set, construct_only)]
        navigation_view: OnceCell<adw::NavigationView>,
        #[property(get, set, construct_only)]
        split_view: OnceCell<adw::NavigationSplitView>,
        #[property(get, set = Self::set_collapsed)]
        collapsed: Cell<bool>,

        #[template_child]
        top_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        middle_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        bottom_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSidebar {
        const NAME: &'static str = "SkSidebar";
        type ParentType = adw::Bin;
        type Type = super::SkSidebar;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkSidebar {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.add_item("discover", "view-grid-symbolic", RowPosition::Top);
            self.add_item("search", "system-search-symbolic", RowPosition::Top);

            self.add_item(
                "create",
                "applications-graphics-symbolic",
                RowPosition::Middle,
            );
            self.add_item("work", "input-keyboard-symbolic", RowPosition::Middle);
            self.add_item("play", "face-monkey-symbolic", RowPosition::Middle);
            self.add_item("socialise", "system-users-symbolic", RowPosition::Middle);
            self.add_item(
                "learn",
                "accessories-dictionary-symbolic",
                RowPosition::Middle,
            );
            self.add_item("develop", "system-run-symbolic", RowPosition::Middle);

            self.add_item("installed", "emblem-default-symbolic", RowPosition::Bottom);
            self.add_item("updates", "view-refresh-symbolic", RowPosition::Bottom);
            self.add_item("account", "user-info-symbolic", RowPosition::Bottom);

            let activated_closure = clone!(@weak self as this => move |_: &gtk::ListBox, row: &gtk::ListBoxRow| {
                let row: &SkSidebarItemRow = row.downcast_ref().unwrap();
                this.obj().navigation_view().replace_with_tags(&[&row.tag()]);
                this.obj().split_view().set_show_content(true);
            });

            self.top_listbox
                .connect_row_activated(activated_closure.clone());
            self.middle_listbox
                .connect_row_activated(activated_closure.clone());
            self.bottom_listbox
                .connect_row_activated(activated_closure.clone());

            self.obj().navigation_view().connect_visible_page_notify(
                clone!(@weak self as this => move |_|{
                    this.update_selection();
                }),
            );

            self.obj()
                .split_view()
                .bind_property("collapsed", &*self.obj(), "collapsed")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
    }

    impl WidgetImpl for SkSidebar {}

    impl BinImpl for SkSidebar {}

    impl SkSidebar {
        fn add_item(&self, tag: &str, icon_name: &str, position: RowPosition) {
            let page = self.obj().navigation_view().find_page(tag).unwrap();
            let row = SkSidebarItemRow::new(&page.title(), tag, icon_name);

            match position {
                RowPosition::Top => self.top_listbox.append(&row),
                RowPosition::Middle => self.middle_listbox.append(&row),
                RowPosition::Bottom => self.bottom_listbox.append(&row),
            }
        }

        fn set_collapsed(&self, collapsed: bool) {
            self.middle_listbox.set_vexpand(!collapsed);

            self.set_listbox_collapsed(&self.top_listbox, collapsed);
            self.set_listbox_collapsed(&self.middle_listbox, collapsed);
            self.set_listbox_collapsed(&self.bottom_listbox, collapsed);

            if !collapsed {
                self.update_selection();
            }

            self.collapsed.set(collapsed);
            self.obj().notify_collapsed();
        }

        fn set_listbox_collapsed(&self, listbox: &gtk::ListBox, collapsed: bool) {
            if collapsed {
                listbox.remove_css_class("navigation-sidebar");
                listbox.add_css_class("boxed-list");

                self.obj().add_css_class("collapsed-sidebar");
                self.obj().remove_css_class("regular-sidebar");

                listbox.set_selection_mode(gtk::SelectionMode::None);
            } else {
                listbox.add_css_class("navigation-sidebar");
                listbox.remove_css_class("boxed-list");

                self.obj().add_css_class("regular-sidebar");
                self.obj().remove_css_class("collapsed-sidebar");

                listbox.set_selection_mode(gtk::SelectionMode::Single);
            }
        }

        fn update_selection(&self) {
            if let Some(page) = self.obj().navigation_view().visible_page() {
                let tag = page.tag().unwrap();
                self.update_listbox_selection(&self.top_listbox, &tag);
                self.update_listbox_selection(&self.middle_listbox, &tag);
                self.update_listbox_selection(&self.bottom_listbox, &tag);
            }
        }

        fn update_listbox_selection(&self, listbox: &gtk::ListBox, selected_tag: &str) {
            listbox.unselect_all();

            let mut index = 0;
            while let Some(row) = listbox.row_at_index(index) {
                let row: &SkSidebarItemRow = row.downcast_ref().unwrap();
                if row.tag() == selected_tag {
                    listbox.select_row(Some(row));
                }
                index += 1;
            }
        }
    }
}

glib::wrapper! {
    pub struct SkSidebar(
        ObjectSubclass<imp::SkSidebar>)
        @extends gtk::Widget, adw::Bin;
}

impl SkSidebar {
    pub fn new(
        navigation_view: &adw::NavigationView,
        split_view: &adw::NavigationSplitView,
    ) -> Self {
        glib::Object::builder()
            .property("navigation-view", navigation_view)
            .property("split-view", split_view)
            .build()
    }
}
