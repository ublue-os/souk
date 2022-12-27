// Souk - transaction_row.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{clone, subclass, ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::OnceCell;

use crate::main::task::SkTask;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/transaction_row.ui")]
    pub struct SkTransactionRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub uuid_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub steps_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,

        pub popover_menu: OnceCell<gtk::PopoverMenu>,
        pub transaction: OnceCell<SkTask>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkTransactionRow {
        const NAME: &'static str = "SkTransactionRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::SkTransactionRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkTransactionRow {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "transaction",
                    "",
                    "",
                    SkTask::static_type(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "transaction" => obj.transaction().to_value(),
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
                "transaction" => self.transaction.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
        }
    }

    impl WidgetImpl for SkTransactionRow {}

    impl ListBoxRowImpl for SkTransactionRow {}
}

glib::wrapper! {
    pub struct SkTransactionRow(
        ObjectSubclass<imp::SkTransactionRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl SkTransactionRow {
    pub fn new(transaction: &SkTask) -> Self {
        glib::Object::new(&[("transaction", transaction)]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        self.bind_property("uuid", imp.uuid_label.get(), "label");
        self.bind_property("progress", imp.progress_bar.get(), "fraction");

        self.transaction().connect_notify_local(
            None,
            clone!(@weak self as this => move |_, _|{
                this.update_labels();
            }),
        );

        self.update_labels();
    }

    pub fn transaction(&self) -> SkTask {
        self.imp().transaction.get().unwrap().clone()
    }

    fn update_labels(&self) {
        let imp = self.imp();
        let transaction = self.transaction();

        // Title
        // let ref_ = transaction.ref_().format_ref().unwrap().to_string();
        let title = format!("{:?}", transaction.type_());
        imp.title_label.set_text(&title);

        // Subtitle
        // if let Some(operation_ref) = transaction.current_operation_ref() {
        // let ref_ = operation_ref.format_ref().unwrap().to_string();
        // let subtitle = format!("{} {}", transaction.current_operation_type(),
        // ref_); imp.subtitle_label.set_text(&subtitle);
        // }
        //
        // Steps label
        //
        // let steps = format!(
        // "{}/{}",
        // transaction.current_operation(),
        // transaction.operations_count()
        // );
        // imp.steps_label.set_text(&steps);
    }

    fn bind_property<T: IsA<gtk::Widget>>(
        &self,
        prop_name: &str,
        widget: T,
        widget_prop_name: &str,
    ) {
        self.transaction()
            .bind_property(prop_name, &widget, widget_prop_name)
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
    }
}
