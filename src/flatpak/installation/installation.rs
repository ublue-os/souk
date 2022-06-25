// Souk - installation.rs
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

use gio::File;
use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::i18n::i18n;
use crate::worker::InstallationInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkInstallation {
        pub uuid: OnceCell<String>,
        pub id: OnceCell<String>,
        pub display_name: OnceCell<String>,
        pub description: OnceCell<String>,
        pub icon_name: OnceCell<String>,
        pub is_user: OnceCell<bool>,
        pub is_custom: OnceCell<bool>,
        pub path: OnceCell<File>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallation {
        const NAME: &'static str = "SkInstallation";
        type ParentType = glib::Object;
        type Type = super::SkInstallation;
    }

    impl ObjectImpl for SkInstallation {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("uuid", "UUID", "UUID", None, ParamFlags::READABLE),
                    ParamSpecString::new("id", "ID", "ID", None, ParamFlags::READABLE),
                    ParamSpecString::new("display-name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("description", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("icon-name", "", "", None, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-user", "", "", false, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-custom", "", "", false, ParamFlags::READABLE),
                    ParamSpecObject::new("path", "", "", File::static_type(), ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uuid" => obj.uuid().to_value(),
                "id" => obj.id().to_value(),
                "display-name" => obj.display_name().to_value(),
                "description" => obj.description().to_value(),
                "icon-name" => obj.icon_name().to_value(),
                "is-user" => obj.is_user().to_value(),
                "is-custom" => obj.is_custom().to_value(),
                "path" => obj.path().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkInstallation(ObjectSubclass<imp::SkInstallation>);
}

impl SkInstallation {
    pub fn new(info: &InstallationInfo) -> Self {
        let installation: Self = glib::Object::new(&[]).unwrap();
        let imp = installation.imp();

        imp.uuid.set(info.uuid.clone()).unwrap();
        imp.id.set(info.id.clone()).unwrap();
        imp.is_user.set(info.is_user).unwrap();
        imp.is_custom.set(info.is_custom).unwrap();
        let path = File::for_parse_name(&info.path);
        imp.path.set(path).unwrap();

        if !info.is_custom && !info.is_user {
            // Default system installation
            let display_name = i18n("System");
            imp.display_name.set(display_name).unwrap();

            let description = i18n("All users on this computer");
            imp.description.set(description).unwrap();

            imp.icon_name.set("people-symbolic".into()).unwrap();
        } else if !info.is_custom && info.is_user {
            // Default user installation
            let display_name = i18n("User");
            imp.display_name.set(display_name).unwrap();

            let description = i18n("Only currently logged in user");
            imp.description.set(description).unwrap();

            imp.icon_name.set("person-symbolic".into()).unwrap();
        } else {
            // Custom installations
            imp.display_name.set(info.display_name.clone()).unwrap();
            imp.description.set(String::new()).unwrap();
            imp.icon_name.set("dotted-box-symbolic".into()).unwrap();
        }

        installation
    }

    pub fn uuid(&self) -> String {
        self.imp().uuid.get().unwrap().to_string()
    }

    pub fn id(&self) -> String {
        self.imp().id.get().unwrap().to_string()
    }

    pub fn display_name(&self) -> String {
        self.imp().display_name.get().unwrap().to_string()
    }

    pub fn description(&self) -> String {
        self.imp().description.get().unwrap().to_string()
    }

    pub fn icon_name(&self) -> String {
        self.imp().icon_name.get().unwrap().to_string()
    }

    pub fn is_user(&self) -> bool {
        *self.imp().is_user.get().unwrap()
    }

    pub fn is_custom(&self) -> bool {
        *self.imp().is_user.get().unwrap()
    }

    pub fn path(&self) -> File {
        self.imp().path.get().unwrap().clone()
    }
}
