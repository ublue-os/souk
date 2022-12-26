// Souk - package.rs
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

use flatpak::prelude::*;
use flatpak::Ref;
use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::SkPackageType;
use crate::shared::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackage {
        pub id: OnceCell<String>,
        pub installation_id: OnceCell<String>,
        pub remote_id: OnceCell<String>,

        pub type_: OnceCell<SkPackageType>,
        pub name: OnceCell<String>,
        pub architecture: OnceCell<String>,
        pub branch: OnceCell<String>,
        pub commit: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackage {
        const NAME: &'static str = "SkPackage";
        type Type = super::SkPackage;
    }

    impl ObjectImpl for SkPackage {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("id", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("installation-id", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("remote-id", "", "", None, ParamFlags::READABLE),
                    ParamSpecEnum::new(
                        "type",
                        "",
                        "",
                        SkPackageType::static_type(),
                        SkPackageType::App as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("architecture", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("branch", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("commit", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "installation-id" => obj.installation_id().to_value(),
                "remote-id" => obj.remote_id().to_value(),
                "type" => obj.type_().to_value(),
                "name" => obj.name().to_value(),
                "architecture" => obj.architecture().to_value(),
                "branch" => obj.branch().to_value(),
                "commit" => obj.commit().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkPackage(ObjectSubclass<imp::SkPackage>);
}

impl SkPackage {
    pub fn new(info: &PackageInfo) -> Self {
        let package: Self = glib::Object::new(&[]).unwrap();
        let imp = package.imp();

        imp.id.set(info.id.clone()).unwrap();
        imp.installation_id
            .set(info.installation_id.clone())
            .unwrap();
        imp.remote_id.set(info.remote_id.clone()).unwrap();

        let flatpak_ref = Ref::parse(&info.ref_).unwrap();

        imp.type_.set(flatpak_ref.kind().into()).unwrap();
        imp.name
            .set(flatpak_ref.name().unwrap().to_string())
            .unwrap();
        imp.architecture
            .set(flatpak_ref.arch().unwrap().to_string())
            .unwrap();
        imp.branch
            .set(flatpak_ref.branch().unwrap().to_string())
            .unwrap();
        imp.commit.set(info.commit.clone()).unwrap();

        package
    }

    pub fn id(&self) -> String {
        self.imp().id.get().unwrap().to_string()
    }

    pub fn installation_id(&self) -> String {
        self.imp().installation_id.get().unwrap().to_string()
    }

    pub fn remote_id(&self) -> String {
        self.imp().remote_id.get().unwrap().to_string()
    }

    pub fn type_(&self) -> SkPackageType {
        *self.imp().type_.get().unwrap()
    }

    pub fn name(&self) -> String {
        self.imp().name.get().unwrap().to_string()
    }

    pub fn architecture(&self) -> String {
        self.imp().architecture.get().unwrap().to_string()
    }

    pub fn branch(&self) -> String {
        self.imp().branch.get().unwrap().clone()
    }

    pub fn commit(&self) -> String {
        self.imp().commit.get().unwrap().clone()
    }
}
