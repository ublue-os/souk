// Souk - installation_row.rs
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

use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{ParamSpec, Properties};
use gtk::{glib, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::main::flatpak::installation::SkInstallation;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::SkInstallationRow)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/installation_row.ui")]
    pub struct SkInstallationRow {
        #[property(get, set, construct_only)]
        installation: OnceCell<SkInstallation>,
        #[property(get, set)]
        selected: Cell<bool>,

        #[template_child]
        checkmark: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationRow {
        const NAME: &'static str = "SkInstallationRow";
        type ParentType = adw::ActionRow;
        type Type = super::SkInstallationRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkInstallationRow {
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
            let installation = self.obj().installation();

            self.obj().set_icon_name(Some(&installation.icon_name()));
            self.obj().set_title(&installation.title());
            self.obj().set_subtitle(&installation.description());

            self.obj()
                .bind_property("selected", &self.checkmark.get(), "visible")
                .build();
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
        glib::Object::builder()
            .property("installation", installation)
            .build()
    }
}
