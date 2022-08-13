// Souk - context_detail_row.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::{subclass, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::OnceCell;

use crate::flatpak::context::{SkContextDetail, SkContextDetailLevel, SkContextDetailType};
use crate::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/context_detail_row.ui")]
    pub struct SkContextDetailRow {
        #[template_child]
        pub type_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub text_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub arrow: TemplateChild<gtk::Image>,

        pub detail: OnceCell<SkContextDetail>,
        pub show_arrow: OnceCell<bool>,
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
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "detail",
                        "",
                        "",
                        SkContextDetail::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecBoolean::new(
                        "show-arrow",
                        "",
                        "",
                        false,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "detail" => obj.detail().to_value(),
                "show-arrow" => obj.show_arrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "detail" => self.detail.set(value.get().unwrap()).unwrap(),
                "show-arrow" => self.show_arrow.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
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
        glib::Object::new(&[("detail", detail), ("show-arrow", &show_arrow)]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let detail = self.detail();

        if detail.type_() == SkContextDetailType::ICON {
            imp.type_stack.set_visible_child(&imp.icon_image.get());
            imp.icon_image.set_icon_name(Some(&detail.type_value()));
        } else if detail.type_() == SkContextDetailType::SIZE {
            imp.type_stack.set_visible_child(&imp.text_label.get());
            let markup = utils::size_to_markup(&detail.type_value());
            imp.text_label.set_markup(&markup);
            imp.text_label.add_css_class("size");
        } else {
            imp.type_stack.set_visible_child(&imp.text_label.get());
            imp.text_label.set_markup(&detail.type_value());
            imp.text_label.remove_css_class("size");
        }

        let css = match detail.level() {
            SkContextDetailLevel::NEUTRAL => "context-neutral",
            SkContextDetailLevel::GOOD => "color-green",
            SkContextDetailLevel::MINOR => "color-blue",
            SkContextDetailLevel::MODERATE => "color-orange",
            SkContextDetailLevel::WARNING => "color-yellow",
            SkContextDetailLevel::BAD => "color-red",
        };
        imp.icon_image.add_css_class(css);
        imp.text_label.add_css_class(css);

        imp.arrow.set_visible(self.show_arrow());

        self.set_title(&detail.title());
        self.set_subtitle(&detail.description());
    }

    pub fn detail(&self) -> SkContextDetail {
        self.imp().detail.get().unwrap().clone()
    }

    pub fn show_arrow(&self) -> bool {
        *self.imp().show_arrow.get().unwrap()
    }
}
