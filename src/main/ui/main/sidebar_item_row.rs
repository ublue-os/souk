// Souk - sidebar_item_row.rs
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

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{ParamSpec, Properties};
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkSidebarItemRow)]
    pub struct SkSidebarItemRow {
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        icon_name: RefCell<String>,
        #[property(get, set)]
        tag: RefCell<String>,

        title_label: gtk::Label,
        icon_image: gtk::Image,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSidebarItemRow {
        const NAME: &'static str = "SkSidebarItemRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::SkSidebarItemRow;
    }

    impl ObjectImpl for SkSidebarItemRow {
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

            let box_ = gtk::Box::builder().build();
            let arrow = gtk::Image::from_icon_name("go-next-symbolic");
            arrow.add_css_class("row-arrow");

            box_.append(&self.icon_image);
            box_.append(&self.title_label);
            box_.append(&arrow);

            self.title_label.set_hexpand(true);
            self.title_label.set_halign(gtk::Align::Start);
            self.title_label
                .set_ellipsize(gtk::pango::EllipsizeMode::End);
            self.obj().set_child(Some(&box_));

            self.obj()
                .bind_property("title", &self.title_label, "label")
                .build();
            self.obj()
                .bind_property("icon_name", &self.icon_image, "icon-name")
                .build();
        }
    }

    impl WidgetImpl for SkSidebarItemRow {}

    impl ListBoxRowImpl for SkSidebarItemRow {}
}

glib::wrapper! {
    pub struct SkSidebarItemRow(
        ObjectSubclass<imp::SkSidebarItemRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl SkSidebarItemRow {
    pub fn new(title: &str, tag: &str, icon_name: &str) -> Self {
        glib::Object::builder()
            .property("title", title)
            .property("tag", tag)
            .property("icon-name", icon_name)
            .build()
    }
}
