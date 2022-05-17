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

use glib::{subclass, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use libflatpak::prelude::*;
use libflatpak::Installation;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::i18n::i18n;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/installation_row.ui")]
    pub struct SkInstallationRow {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        pub checkmark: TemplateChild<gtk::Image>,

        pub installation: OnceCell<Installation>,
        pub installation_title: OnceCell<String>,
        pub selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationRow {
        const NAME: &'static str = "SkInstallationRow";
        type ParentType = gtk::ListBoxRow;
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
                        "Installation",
                        "Installation",
                        Installation::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new(
                        "installation-title",
                        "Installation Title",
                        "Installation Title",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new(
                        "selected",
                        "Selected",
                        "Selected",
                        false,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "installation" => obj.installation().to_value(),
                "installation-title" => obj.installation_title().to_value(),
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
}

glib::wrapper! {
    pub struct SkInstallationRow(
        ObjectSubclass<imp::SkInstallationRow>)
        @extends gtk::Widget, gtk::ListBoxRow;

}

impl SkInstallationRow {
    pub fn new(installation: &Installation) -> Self {
        glib::Object::new(&[("installation", installation)]).unwrap()
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let installation = self.installation();

        // Overwrites for known Flatpak installations
        let title = match installation.id().unwrap().as_str() {
            "default" => {
                imp.icon.set_icon_name(Some("people-symbolic"));
                imp.subtitle.set_label(&i18n("Available for all users"));
                i18n("System")
            }
            "user" => {
                imp.icon.set_icon_name(Some("person-symbolic"));
                imp.subtitle.set_label(&i18n("Only for the current user"));
                i18n("User")
            }
            _ => {
                imp.subtitle.set_label(&i18n("No description available"));
                installation.id().unwrap().to_string()
            }
        };

        imp.title.set_label(&title);
        imp.installation_title.set(title).unwrap();
        self.notify("installation-title");

        self.bind_property("selected", &imp.checkmark.get(), "visible")
            .build();
    }

    pub fn installation(&self) -> Installation {
        self.imp().installation.get().unwrap().clone()
    }

    pub fn installation_title(&self) -> String {
        self.imp().installation_title.get().unwrap().clone()
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().selected.set(selected);
        self.notify("selected");
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }
}
