// Souk - context_box.rs
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
use glib::{clone, subclass, ParamSpec, Properties};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::main::context::{
    SkContext, SkContextDetail, SkContextDetailGroup, SkContextDetailKind, SkContextDetailLevel,
};
use crate::main::ui::context::SkContextDetailRow;
use crate::main::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SkContextBox)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/context_box.ui")]
    pub struct SkContextBox {
        #[property(get, set = Self::set_context)]
        context: RefCell<Option<SkContext>>,

        #[template_child]
        kind_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        text_label: TemplateChild<gtk::Label>,

        #[template_child]
        title_label: TemplateChild<gtk::Label>,
        #[template_child]
        description_label: TemplateChild<gtk::Label>,
        #[template_child]
        details_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContextBox {
        const NAME: &'static str = "SkContextBox";
        type ParentType = gtk::Box;
        type Type = super::SkContextBox;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            // Register gobject type so that it can get used in context_box.ui builder xml
            SkContext::static_type();

            obj.init_template();
        }
    }

    impl ObjectImpl for SkContextBox {
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

    impl WidgetImpl for SkContextBox {}

    impl BoxImpl for SkContextBox {}

    impl SkContextBox {
        fn set_context(&self, value: &SkContext) {
            *self.context.borrow_mut() = Some(value.clone());
            self.obj().notify("context");
            self.update_widgets();
        }

        fn update_widgets(&self) {
            let context = self.obj().context().unwrap();
            let summary = context.summary();

            if summary.kind() == SkContextDetailKind::Icon {
                self.kind_stack.set_visible_child(&self.icon_image.get());
                self.icon_image.set_icon_name(Some(&summary.kind_value()));
            } else if summary.kind() == SkContextDetailKind::Size {
                let markup = utils::size_to_markup(&summary.kind_value());
                self.text_label.set_markup(&markup);
            } else {
                self.kind_stack.set_visible_child(&self.text_label.get());
                self.text_label.set_markup(&summary.kind_value());
            }

            // Remove previous set css classes
            utils::remove_css_colors(&self.icon_image.get());
            utils::remove_css_colors(&self.text_label.get());

            let css = match summary.level() {
                SkContextDetailLevel::Neutral => "color-neutral",
                SkContextDetailLevel::Good => "color-good",
                SkContextDetailLevel::Minor => "color-minor",
                SkContextDetailLevel::Moderate => "color-moderate",
                SkContextDetailLevel::Warning => "color-warning",
                SkContextDetailLevel::Bad => "color-bad",
            };
            self.icon_image.add_css_class(css);
            self.text_label.add_css_class(css);

            self.title_label.set_text(&context.summary().title());
            self.description_label
                .set_markup(&context.summary().description());

            self.details_listbox.bind_model(
                Some(&context.details()),
                clone!(@weak self as this => @default-panic, move |detail_group| {
                    let detail_group = detail_group.downcast_ref::<SkContextDetailGroup>().unwrap();

                    let group_box = adw::PreferencesGroup::new();
                    group_box.set_margin_bottom(12);

                    if let Some(title) = detail_group.title(){
                        group_box.set_title(&title);
                    }
                    if let Some(description) = detail_group.description(){
                        group_box.set_description(Some(&description));
                    }

                    for detail in detail_group.snapshot().iter(){
                        let detail = detail.downcast_ref::<SkContextDetail>().unwrap();
                        group_box.add(&SkContextDetailRow::new(detail, false));
                    }

                    let row = gtk::ListBoxRow::new();
                    row.set_child(Some(&group_box));
                    row.set_activatable(false);

                    row.upcast()
                }),
            );
        }
    }
}

glib::wrapper! {
    pub struct SkContextBox(
        ObjectSubclass<imp::SkContextBox>)
        @extends gtk::Widget, gtk::Box;
}

impl SkContextBox {
    pub fn new(context: &SkContext) -> Self {
        glib::Object::builder().property("context", context).build()
    }
}
