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

use glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::SkInstallation;
use crate::main::SkApplication;
use crate::shared::info::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkRemote {
        pub info: OnceCell<RemoteInfo>,
        pub installation: OnceCell<Option<SkInstallation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemote {
        const NAME: &'static str = "SkRemote";
        type Type = super::SkRemote;
    }

    impl ObjectImpl for SkRemote {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("repository-url", "", "", None, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "installation",
                        "",
                        "",
                        SkInstallation::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("title", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("description", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("comment", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("homepage", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("icon", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.obj().name().to_value(),
                "repository-url" => self.obj().repository_url().to_value(),
                "installation" => self.obj().installation().to_value(),
                "title" => self.obj().title().to_value(),
                "description" => self.obj().description().to_value(),
                "comment" => self.obj().comment().to_value(),
                "homepage" => self.obj().homepage().to_value(),
                "icon" => self.obj().icon().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkRemote(ObjectSubclass<imp::SkRemote>);
}

impl SkRemote {
    pub fn new(info: &RemoteInfo) -> Self {
        let remote: Self = glib::Object::new(&[]);
        let imp = remote.imp();

        imp.info.set(info.clone()).unwrap();

        if let Some(inst_info) = &info.installation.clone().into() {
            let installations = SkApplication::default().worker().installations();
            let installation = installations
                .installation(inst_info)
                .expect("Unknown Flatpak installation");
            imp.installation.set(Some(installation)).unwrap();
        } else {
            imp.installation.set(None).unwrap();
        }

        remote
    }

    pub fn name(&self) -> String {
        self.info().name
    }

    pub fn repository_url(&self) -> String {
        self.info().repository_url
    }

    pub fn installation(&self) -> Option<SkInstallation> {
        self.imp().installation.get().unwrap().clone()
    }

    pub fn title(&self) -> String {
        self.info().title
    }

    pub fn description(&self) -> String {
        self.info().description
    }

    pub fn comment(&self) -> String {
        self.info().comment
    }

    pub fn homepage(&self) -> String {
        self.info().homepage
    }

    pub fn icon(&self) -> String {
        self.info().icon
    }

    pub fn info(&self) -> RemoteInfo {
        self.imp().info.get().unwrap().clone()
    }
}
