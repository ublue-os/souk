// Souk - context_box.rs
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

use std::cell::RefCell;

use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::flatpak::context::{
    SkContext, SkContextDetail, SkContextDetailGroup, SkContextDetailType,
};
use crate::ui::context::SkContextDetailRow;
use crate::ui::utils;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/context_box.ui")]
    pub struct SkContextBox {
        #[template_child]
        pub type_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub text_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_listbox: TemplateChild<gtk::ListBox>,

        pub context: RefCell<Option<SkContext>>,
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
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "context",
                    "",
                    "",
                    SkContextDetail::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "context" => obj.context().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "context" => obj.set_context(&value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SkContextBox {}

    impl BoxImpl for SkContextBox {}
}

glib::wrapper! {
    pub struct SkContextBox(
        ObjectSubclass<imp::SkContextBox>)
        @extends gtk::Widget, gtk::Box;
}

impl SkContextBox {
    pub fn new(context: &SkContext) -> Self {
        glib::Object::new(&[("context", context)]).unwrap()
    }

    pub fn context(&self) -> SkContext {
        self.imp().context.borrow().as_ref().unwrap().clone()
    }

    pub fn set_context(&self, value: &SkContext) {
        *self.imp().context.borrow_mut() = Some(value.clone());
        self.notify("context");
        self.update_widgets();
    }

    fn update_widgets(&self) {
        let imp = self.imp();
        let context = self.context();
        let summary = context.summary();

        if summary.type_() == SkContextDetailType::Icon {
            imp.type_stack.set_visible_child(&imp.icon_image.get());
            imp.icon_image.set_icon_name(Some(&summary.type_value()));
        } else if summary.type_() == SkContextDetailType::Size {
            let markup = utils::size_to_markup(&summary.type_value());
            imp.text_label.set_markup(&markup);
        } else {
            imp.type_stack.set_visible_child(&imp.text_label.get());
            imp.text_label.set_markup(&summary.type_value());
        }

        imp.title_label.set_text(&context.summary().title());
        imp.description_label
            .set_text(&context.summary().description());

        imp.details_listbox
            .bind_model(Some(&context.details()), clone!(@weak self as this => @default-panic, move |detail_group| {
                let context_detail_group = detail_group.downcast_ref::<SkContextDetailGroup>().unwrap();

                let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
                if let Some(descr) = context_detail_group.description(){
                    let label = gtk::Label::new(Some(&descr));
                    label.set_xalign(0.0);
                    label.set_wrap(true);
                    label.add_css_class("dim-label");
                    box_.append(&label);
                }

                let listbox = gtk::ListBox::new();
                listbox.add_css_class("content");
                listbox.set_margin_bottom(18);
                listbox.set_selection_mode(gtk::SelectionMode::None);
                listbox.bind_model(Some(context_detail_group), |detail|{
                    let detail = detail.downcast_ref::<SkContextDetail>().unwrap();
                    SkContextDetailRow::new(detail, false).upcast()
                });
                box_.append(&listbox);

                let row = gtk::ListBoxRow::new();
                row.set_child(Some(&box_));
                row.set_activatable(false);

                row.upcast()
            }));
    }
}
