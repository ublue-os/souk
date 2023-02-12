// Souk - badge.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::{subclass, ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecString};
use gtk::{glib, CompositeTemplate};

use crate::main::ui::badge::SkBadgeType;
use crate::main::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/badge.ui")]
    pub struct SkBadge {
        #[template_child]
        pub image: TemplateChild<gtk::Image>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub type_: RefCell<SkBadgeType>,
        pub value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkBadge {
        const NAME: &'static str = "SkBadge";
        type ParentType = adw::Bin;
        type Type = super::SkBadge;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkBadge {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkBadgeType::static_type(),
                        SkBadgeType::default() as i32,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new("value", "", "", None, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => self.obj().type_().to_value(),
                "value" => self.obj().value().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "type" => self.obj().set_type(value.get().unwrap()),
                "value" => self.obj().set_value(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SkBadge {}

    impl BinImpl for SkBadge {}
}

glib::wrapper! {
    pub struct SkBadge(
        ObjectSubclass<imp::SkBadge>)
        @extends gtk::Widget, adw::Bin;
}

impl SkBadge {
    pub fn new(type_: SkBadgeType, value: &str) -> Self {
        glib::Object::builder()
            .property("type", &type_)
            .property("value", &value)
            .build()
    }

    pub fn type_(&self) -> SkBadgeType {
        *self.imp().type_.borrow()
    }

    pub fn set_type(&self, type_: SkBadgeType) {
        *self.imp().type_.borrow_mut() = type_;
        self.notify("type");

        self.update_icon();
    }

    pub fn value(&self) -> String {
        self.imp().value.borrow().clone()
    }

    pub fn set_value(&self, value: &str) {
        let text = value.to_uppercase();
        self.imp().label.set_text(&text);

        *self.imp().value.borrow_mut() = value.into();
        self.notify("value");

        self.update_icon();
    }

    fn update_icon(&self) {
        let icon = match self.type_() {
            SkBadgeType::Repository => "repo-symbolic",
            SkBadgeType::Branch => match self.value().to_lowercase().as_str() {
                "stable" => "branch-stable-symbolic",
                "beta" => "branch-beta-symbolic",
                "master" | "nightly" | "daily" => "branch-unstable-symbolic",
                _ => "branch-generic-symbolic",
            },
        };

        self.imp().image.set_icon_name(Some(icon));
        self.update_css();
    }

    fn update_css(&self) {
        utils::remove_css_colors(self);

        let css = match self.type_() {
            SkBadgeType::Repository => "color-blue",
            SkBadgeType::Branch => match self.value().to_lowercase().as_str() {
                "stable" => "color-green",
                "beta" => "color-orange",
                "master" | "nightly" | "daily" => "color-red",
                _ => "color-neutral",
            },
        };

        self.add_css_class(css);
    }
}
