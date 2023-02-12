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
use glib::{ParamSpec, Properties};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use super::{SkPackageKind, SkPackageSubrefKind};
use crate::main::flatpak::installation::SkRemote;
use crate::main::SkApplication;
use crate::shared::flatpak::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkPackage)]
    pub struct SkPackage {
        #[property(get)]
        pub remote: OnceCell<SkRemote>,
        #[property(name = "kind", get = Self::kind, type = SkPackageKind, builder(SkPackageKind::App))]
        #[property(name = "subref-kind", get = Self::subref_kind, type = SkPackageSubrefKind, builder(SkPackageSubrefKind::None))]
        #[property(name = "name", get = Self::name, type = String)]
        #[property(name = "architecture", get = Self::architecture, type = String)]
        #[property(name = "branch", get = Self::branch, type = String)]
        #[property(get, set, construct_only)]
        pub info: OnceCell<PackageInfo>,

        pub flatpak_ref: OnceCell<Ref>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackage {
        const NAME: &'static str = "SkPackage";
        type Type = super::SkPackage;
    }

    impl ObjectImpl for SkPackage {
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

    impl SkPackage {
        fn kind(&self) -> SkPackageKind {
            self.flatpak_ref.get().unwrap().kind().into()
        }

        fn subref_kind(&self) -> SkPackageSubrefKind {
            SkPackageSubrefKind::from(self.obj().name().as_str())
        }

        fn name(&self) -> String {
            self.flatpak_ref.get().unwrap().name().unwrap().to_string()
        }

        fn architecture(&self) -> String {
            self.flatpak_ref.get().unwrap().arch().unwrap().to_string()
        }

        fn branch(&self) -> String {
            self.flatpak_ref
                .get()
                .unwrap()
                .branch()
                .unwrap()
                .to_string()
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
    fn kind(&self) -> SkPackageKind {
        self.upcast_ref().kind()
    }

    fn subref_kind(&self) -> SkPackageSubrefKind {
        self.upcast_ref().subref_kind()
    }

    fn name(&self) -> String {
        self.upcast_ref().name()
    }

    fn architecture(&self) -> String {
        self.upcast_ref().architecture()
    }

    fn branch(&self) -> String {
        self.upcast_ref().branch()
    }

    fn remote(&self) -> SkRemote {
        self.upcast_ref().remote()
    }

    fn info(&self) -> PackageInfo {
        self.upcast_ref().info()
    }
}

pub trait SkPackageImpl: ObjectImpl {}

/// Make SkPackage subclassable
unsafe impl<T: SkPackageImpl> IsSubclassable<T> for SkPackage {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
