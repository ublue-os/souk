// Souk - context_detail_row.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{subclass, ParamSpec, Properties};
use gtk::{glib, CompositeTemplate};
use once_cell::sync::OnceCell;

use crate::main::context::{SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SkContextDetailRow)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/context_detail_row.ui")]
    pub struct SkContextDetailRow {
        #[property(get, set, construct_only)]
        detail: OnceCell<SkContextDetail>,
        #[property(get, set, construct_only)]
        show_arrow: OnceCell<bool>,

        #[template_child]
        kind_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        text_label: TemplateChild<gtk::Label>,
        #[template_child]
        arrow: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextDetailRow {
        const NAME: &'static str = "SkContextDetailRow";
        type ParentType = adw::ActionRow;
        type Type = super::SkContextDetailRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkContextDetailRow {
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

            let detail = self.obj().detail();

            if detail.kind() == SkContextDetailKind::Icon {
                self.kind_stack.set_visible_child(&self.icon_image.get());
                self.icon_image.set_icon_name(Some(&detail.kind_value()));
            } else if detail.kind() == SkContextDetailKind::Size {
                self.kind_stack.set_visible_child(&self.text_label.get());
                let markup = utils::size_to_markup(&detail.kind_value());
                self.text_label.set_markup(&markup);
                self.text_label.add_css_class("size");
            } else {
                self.kind_stack.set_visible_child(&self.text_label.get());
                self.text_label.set_markup(&detail.kind_value());
                self.text_label.remove_css_class("size");
            }

            let css = match detail.level() {
                SkContextDetailLevel::Neutral => "color-neutral",
                SkContextDetailLevel::Good => "color-green",
                SkContextDetailLevel::Minor => "color-blue",
                SkContextDetailLevel::Moderate => "color-orange",
                SkContextDetailLevel::Warning => "color-yellow",
                SkContextDetailLevel::Bad => "color-red",
            };
            self.icon_image.add_css_class(css);
            self.text_label.add_css_class(css);

            self.arrow.set_visible(self.obj().show_arrow());

            self.obj().set_title(&detail.title());
            self.obj().set_subtitle(&detail.description());
        }
    }

    impl WidgetImpl for SkContextDetailRow {}

    impl ListBoxRowImpl for SkContextDetailRow {}

    impl PreferencesRowImpl for SkContextDetailRow {}

    impl ActionRowImpl for SkContextDetailRow {}
}

glib::wrapper! {
    pub struct SkContextDetailRow(
        ObjectSubclass<imp::SkContextDetailRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl SkContextDetailRow {
    pub fn new(detail: &SkContextDetail, show_arrow: bool) -> Self {
        glib::Object::builder()
            .property("detail", detail)
            .property("show-arrow", &show_arrow)
            .build()
    }
}
