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
use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, ParamSpecString, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::SkPackageType;
use crate::main::flatpak::installation::SkRemote;
use crate::main::SkApplication;
use crate::shared::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackage {
        pub info: OnceCell<PackageInfo>,

        pub type_: OnceCell<SkPackageType>,
        pub name: OnceCell<String>,
        pub architecture: OnceCell<String>,
        pub branch: OnceCell<String>,
        pub remote: OnceCell<SkRemote>,
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
                    ParamSpecObject::new(
                        "remote",
                        "",
                        "",
                        SkRemote::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "name" => obj.name().to_value(),
                "architecture" => obj.architecture().to_value(),
                "branch" => obj.branch().to_value(),
                "remote" => obj.remote().to_value(),
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

        imp.info.set(info.clone()).unwrap();

        let flatpak_ref = Ref::parse(&info.ref_).unwrap();
        imp.type_.set(flatpak_ref.kind().into()).unwrap();

        let name = flatpak_ref.name().unwrap().to_string();
        imp.name.set(name).unwrap();

        let architecture = flatpak_ref.arch().unwrap().to_string();
        imp.architecture.set(architecture).unwrap();

        let branch = flatpak_ref.branch().unwrap().to_string();
        imp.branch.set(branch).unwrap();

        if let Some(inst_info) = &info.remote.installation.clone().into() {
            let installations = SkApplication::default().worker().installations();
            let installation = installations
                .installation(inst_info)
                .expect("Unknown Flatpak installation");

            let remote = installation.remotes().remote(&info.remote);

            if let Some(remote) = remote {
                imp.remote.set(remote).unwrap();
            } else {
                error!("Flatpak remote {} is not available in installations model, unable to reuse remote object", info.remote.name);
                imp.remote.set(SkRemote::new(&info.remote)).unwrap();
            }
        } else {
            let remote = SkRemote::new(&info.remote);
            imp.remote.set(remote).unwrap();
        }

        package
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

    pub fn remote(&self) -> SkRemote {
        self.imp().remote.get().unwrap().clone()
    }

    pub fn info(&self) -> PackageInfo {
        self.imp().info.get().unwrap().clone()
    }
}
