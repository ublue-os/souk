// Souk - remote.rs
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

use super::SkInstallation;
use crate::main::SkApplication;
use crate::shared::flatpak::info::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkRemote)]
    pub struct SkRemote {
        #[property(get)]
        installation: OnceCell<Option<SkInstallation>>,
        #[property(name = "name", get, type = String, member = name)]
        #[property(name = "repository-url", get, type = String, member = repository_url)]
        #[property(name = "title", get, type = String, member = title)]
        #[property(name = "description", get, type = String, member = description)]
        #[property(name = "comment", get, type = String, member = comment)]
        #[property(name = "homepage", get, type = String, member = homepage)]
        #[property(name = "icon", get, type = String, member = icon)]
        #[property(get, set, construct_only)]
        info: OnceCell<RemoteInfo>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemote {
        const NAME: &'static str = "SkRemote";
        type Type = super::SkRemote;
    }

    impl ObjectImpl for SkRemote {
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

            let info = self.obj().info();

            if let Some(inst_info) = &info.installation.into() {
                let installations = SkApplication::default().worker().installations();
                let installation = installations
                    .installation(inst_info)
                    .expect("Unknown Flatpak installation");
                self.installation.set(Some(installation)).unwrap();
            } else {
                self.installation.set(None).unwrap();
            }
        }
    }
}

glib::wrapper! {
    pub struct SkRemote(ObjectSubclass<imp::SkRemote>);
}

impl SkRemote {
    pub fn new(info: &RemoteInfo) -> Self {
        glib::Object::builder().property("info", &info).build()
    }
}
