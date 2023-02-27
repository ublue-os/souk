// Souk - context_detail.rs
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

use glib::{ParamSpec, Properties};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use crate::main::context::{SkContextDetailKind, SkContextDetailLevel};

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkContextDetail)]
    pub struct SkContextDetail {
        #[property(get, set, construct_only, builder(SkContextDetailKind::Icon))]
        kind: OnceCell<SkContextDetailKind>,
        #[property(get, set, construct_only)]
        kind_value: OnceCell<String>,
        #[property(get, set, construct_only, builder(SkContextDetailLevel::Neutral))]
        level: OnceCell<SkContextDetailLevel>,
        #[property(get, set, construct_only)]
        title: OnceCell<String>,
        #[property(get, set, construct_only)]
        description: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetail {
        const NAME: &'static str = "SkContextDetail";
        type Type = super::SkContextDetail;
    }

    impl ObjectImpl for SkContextDetail {
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
}

glib::wrapper! {
    pub struct SkContextDetail(ObjectSubclass<imp::SkContextDetail>);
}

impl SkContextDetail {
    pub fn new(
        kind: SkContextDetailKind,
        kind_value: &str,
        level: SkContextDetailLevel,
        title: &str,
        description: &str,
    ) -> Self {
        glib::Object::builder()
            .property("kind", kind)
            .property("kind-value", kind_value)
            .property("level", level)
            .property("title", title)
            .property("description", description)
            .build()
    }

    pub fn new_neutral_size(size: u64, title: &str, description: &str) -> Self {
        glib::Object::builder()
            .property("kind", SkContextDetailKind::Size)
            .property("kind-value", &size.to_string())
            .property("level", SkContextDetailLevel::Neutral)
            .property("title", title)
            .property("description", description)
            .build()
    }

    pub fn new_neutral_text(text: &str, title: &str, description: &str) -> Self {
        glib::Object::builder()
            .property("kind", SkContextDetailKind::Text)
            .property("kind-value", &text.to_string())
            .property("level", SkContextDetailLevel::Neutral)
            .property("title", title)
            .property("description", description)
            .build()
    }
}
