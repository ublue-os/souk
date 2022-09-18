// Souk - installation_row.rs
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

use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{subclass, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::installation::SkInstallation;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/installation_row.ui")]
    pub struct SkInstallationRow {
        #[template_child]
        pub checkmark: TemplateChild<gtk::Image>,

        pub installation: OnceCell<SkInstallation>,
        pub selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationRow {
        const NAME: &'static str = "SkInstallationRow";
        type ParentType = adw::ActionRow;
        type Type = super::SkInstallationRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkInstallationRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "installation",
                        "",
                        "",
                        SkInstallation::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecBoolean::new("selected", "", "Selected", false, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "installation" => obj.installation().to_value(),
                "selected" => obj.selected().to_value(),
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
                "installation" => self.installation.set(value.get().unwrap()).unwrap(),
                "selected" => obj.set_selected(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_widgets();
        }
    }

    impl WidgetImpl for SkInstallationRow {}

    impl ListBoxRowImpl for SkInstallationRow {}

    impl PreferencesRowImpl for SkInstallationRow {}

    impl ActionRowImpl for SkInstallationRow {}
}

glib::wrapper! {
    pub struct SkInstallationRow(
        ObjectSubclass<imp::SkInstallationRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;

}

impl SkInstallationRow {
    pub fn new(installation: &SkInstallation) -> Self {
        glib::Object::new(&[("installation", installation)]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let installation = self.installation();

        self.set_icon_name(Some(&installation.icon_name()));
        self.set_title(&installation.title());
        self.set_subtitle(&installation.description());

        self.bind_property("selected", &imp.checkmark.get(), "visible")
            .build();
    }

    pub fn installation(&self) -> SkInstallation {
        self.imp().installation.get().unwrap().clone()
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().selected.set(selected);
        self.notify("selected");
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }
}
