// Souk - package.rs
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

use flatpak::prelude::*;
use flatpak::Ref;
use glib::{
    ParamFlags, ParamSpec, ParamSpecBoxed, ParamSpecEnum, ParamSpecObject, ParamSpecString, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::{SkPackageKind, SkPackageSubrefKind};
use crate::main::flatpak::installation::SkRemote;
use crate::main::SkApplication;
use crate::shared::flatpak::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackage {
        pub info: OnceCell<PackageInfo>,
        pub flatpak_ref: OnceCell<Ref>,
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
                    ParamSpecBoxed::new(
                        "info",
                        "",
                        "",
                        PackageInfo::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecEnum::new(
                        "kind",
                        "",
                        "",
                        SkPackageKind::static_type(),
                        SkPackageKind::App as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "subref-kind",
                        "",
                        "",
                        SkPackageKind::static_type(),
                        SkPackageKind::App as i32,
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

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "info" => self.obj().info().to_value(),
                "kind" => self.obj().kind().to_value(),
                "subref-kind" => self.obj().kind().to_value(),
                "name" => self.obj().name().to_value(),
                "architecture" => self.obj().architecture().to_value(),
                "branch" => self.obj().branch().to_value(),
                "remote" => self.obj().remote().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "info" => self.info.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let info = self.obj().info();

            let flatpak_ref = Ref::parse(&info.ref_).unwrap();
            self.flatpak_ref.set(flatpak_ref).unwrap();

            if let Some(inst_info) = &info.remote.installation.clone().into() {
                let installations = SkApplication::default().worker().installations();
                let installation = installations
                    .installation(inst_info)
                    .expect("Unknown Flatpak installation");

                let remote = installation.remotes().remote(&info.remote);

                if let Some(remote) = remote {
                    self.remote.set(remote).unwrap();
                } else {
                    error!("Flatpak remote {} is not available in installations model, unable to reuse remote object", info.remote.name);
                    self.remote.set(SkRemote::new(&info.remote)).unwrap();
                }
            } else {
                let remote = SkRemote::new(&info.remote);
                self.remote.set(remote).unwrap();
            }
        }
    }
}

glib::wrapper! {
    pub struct SkPackage(ObjectSubclass<imp::SkPackage>);
}

impl SkPackage {
    pub fn new(info: &PackageInfo) -> Self {
        glib::Object::builder().property("info", &info).build()
    }
}

pub trait SkPackageExt: 'static {
    fn info(&self) -> PackageInfo;

    fn kind(&self) -> SkPackageKind;

    fn subref_kind(&self) -> SkPackageSubrefKind;

    fn name(&self) -> String;

    fn architecture(&self) -> String;

    fn branch(&self) -> String;

    fn remote(&self) -> SkRemote;
}

impl<O: IsA<SkPackage>> SkPackageExt for O {
    fn info(&self) -> PackageInfo {
        let obj = self.upcast_ref();
        obj.imp().info.get().unwrap().clone()
    }

    fn kind(&self) -> SkPackageKind {
        let obj = self.upcast_ref();
        obj.imp().flatpak_ref.get().unwrap().kind().into()
    }

    fn subref_kind(&self) -> SkPackageSubrefKind {
        SkPackageSubrefKind::from(self.upcast_ref().name().as_str())
    }

    fn name(&self) -> String {
        let obj = self.upcast_ref();
        obj.imp()
            .flatpak_ref
            .get()
            .unwrap()
            .name()
            .unwrap()
            .to_string()
    }

    fn architecture(&self) -> String {
        let obj = self.upcast_ref();
        obj.imp()
            .flatpak_ref
            .get()
            .unwrap()
            .arch()
            .unwrap()
            .to_string()
    }

    fn branch(&self) -> String {
        let obj = self.upcast_ref();
        obj.imp()
            .flatpak_ref
            .get()
            .unwrap()
            .branch()
            .unwrap()
            .to_string()
    }

    fn remote(&self) -> SkRemote {
        let obj = self.upcast_ref();
        obj.imp().remote.get().unwrap().clone()
    }
}

pub trait SkPackageImpl: ObjectImpl {}

/// Make SkPackage subclassable
unsafe impl<T: SkPackageImpl> IsSubclassable<T> for SkPackage {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
