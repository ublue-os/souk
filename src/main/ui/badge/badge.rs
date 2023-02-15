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
use glib::{ParamSpec, Properties};
use gtk::{glib, CompositeTemplate};

use crate::main::ui::badge::SkBadgeKind;
use crate::main::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SkBadge)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/badge.ui")]
    pub struct SkBadge {
        #[property(get, set = Self::set_kind, builder(SkBadgeKind::Branch))]
        kind: RefCell<SkBadgeKind>,
        #[property(get, set = Self::set_value)]
        value: RefCell<String>,

        #[template_child]
        image: TemplateChild<gtk::Image>,
        #[template_child]
        label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkBadge {
        const NAME: &'static str = "SkBadge";
        type ParentType = adw::Bin;
        type Type = super::SkBadge;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkBadge {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
    }

    impl WidgetImpl for SkBadge {}

    impl BinImpl for SkBadge {}

    impl SkBadge {
        fn set_kind(&self, kind: SkBadgeKind) {
            *self.kind.borrow_mut() = kind;
            self.obj().notify("kind");

            self.update_icon();
        }

        fn set_value(&self, value: &str) {
            let text = value.to_uppercase();
            self.label.set_text(&text);

            *self.value.borrow_mut() = value.into();
            self.obj().notify("value");

            self.update_icon();
        }

        fn update_icon(&self) {
            let icon = match self.obj().kind() {
                SkBadgeKind::Repository => "repo-symbolic",
                SkBadgeKind::Branch => match self.obj().value().to_lowercase().as_str() {
                    "stable" => "branch-stable-symbolic",
                    "beta" => "branch-beta-symbolic",
                    "master" | "nightly" | "daily" => "branch-unstable-symbolic",
                    _ => "branch-generic-symbolic",
                },
            };

            self.image.set_icon_name(Some(icon));
            self.update_css();
        }

        fn update_css(&self) {
            utils::remove_css_colors(&self.obj().clone());

            let css = match self.obj().kind() {
                SkBadgeKind::Repository => "color-blue",
                SkBadgeKind::Branch => match self.obj().value().to_lowercase().as_str() {
                    "stable" => "color-green",
                    "beta" => "color-orange",
                    "master" | "nightly" | "daily" => "color-red",
                    _ => "color-neutral",
                },
            };

            self.obj().add_css_class(css);
        }
    }
}

glib::wrapper! {
    pub struct SkBadge(
        ObjectSubclass<imp::SkBadge>)
        @extends gtk::Widget, adw::Bin;
}

impl SkBadge {
    pub fn new(kind: SkBadgeKind, value: &str) -> Self {
        glib::Object::builder()
            .property("kind", &kind)
            .property("value", &value)
            .build()
    }
}
