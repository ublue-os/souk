// Souk - remote.rs
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

use glib::{ParamFlags, ParamSpec, ParamSpecString, ToValue};
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::worker::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkRemote {
        pub id: OnceCell<String>,
        pub installation_id: OnceCell<String>,
        pub name: OnceCell<String>,
        pub repository_url: OnceCell<String>,

        pub title: OnceCell<Option<String>>,
        pub description: OnceCell<Option<String>>,
        pub comment: OnceCell<Option<String>>,
        pub homepage: OnceCell<Option<String>>,
        pub icon: OnceCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemote {
        const NAME: &'static str = "SkRemote";
        type ParentType = glib::Object;
        type Type = super::SkRemote;
    }

    impl ObjectImpl for SkRemote {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("id", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("installation-id", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("repository-url", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("title", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("description", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("comment", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("homepage", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("icon", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "installation-id" => obj.installation_id().to_value(),
                "name" => obj.name().to_value(),
                "repository-url" => obj.repository_url().to_value(),
                "title" => obj.title().to_value(),
                "description" => obj.description().to_value(),
                "comment" => obj.comment().to_value(),
                "homepage" => obj.homepage().to_value(),
                "icon" => obj.icon().to_value(),
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
        let remote: Self = glib::Object::new(&[]).unwrap();
        let imp = remote.imp();

        imp.id.set(info.id.clone()).unwrap();
        imp.installation_id
            .set(info.installation_id.clone())
            .unwrap();
        imp.name.set(info.name.clone()).unwrap();
        imp.repository_url.set(info.repository_url.clone()).unwrap();

        imp.title.set(info.title.clone().into()).unwrap();
        imp.description
            .set(info.description.clone().into())
            .unwrap();
        imp.comment.set(info.comment.clone().into()).unwrap();
        imp.homepage.set(info.homepage.clone().into()).unwrap();
        imp.icon.set(info.icon.clone().into()).unwrap();

        remote
    }

    pub fn id(&self) -> String {
        self.imp().id.get().unwrap().to_string()
    }

    pub fn installation_id(&self) -> String {
        self.imp().installation_id.get().unwrap().to_string()
    }

    pub fn name(&self) -> String {
        self.imp().name.get().unwrap().to_string()
    }

    pub fn repository_url(&self) -> String {
        self.imp().repository_url.get().unwrap().to_string()
    }

    pub fn title(&self) -> Option<String> {
        self.imp().title.get().unwrap().clone()
    }

    pub fn description(&self) -> Option<String> {
        self.imp().description.get().unwrap().clone()
    }

    pub fn comment(&self) -> Option<String> {
        self.imp().comment.get().unwrap().clone()
    }

    pub fn homepage(&self) -> Option<String> {
        self.imp().homepage.get().unwrap().clone()
    }

    pub fn icon(&self) -> Option<String> {
        self.imp().icon.get().unwrap().clone()
    }
}
