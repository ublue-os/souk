// Souk - dry_run_runtime.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{
    KeyFile, KeyFileFlags, ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, ParamSpecUInt64,
    ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::package::{SkPackage, SkPackageAppstream};
use crate::main::flatpak::SkFlatpakOperationType;
use crate::shared::flatpak::dry_run::DryRunRuntime;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRunRuntime {
        pub data: OnceCell<DryRunRuntime>,
        pub package: OnceCell<SkPackage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRunRuntime {
        const NAME: &'static str = "SkDryRunRuntime";
        type Type = super::SkDryRunRuntime;
    }

    impl ObjectImpl for SkDryRunRuntime {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "package",
                        "",
                        "",
                        SkPackage::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "operation-type",
                        "",
                        "",
                        SkFlatpakOperationType::static_type(),
                        SkFlatpakOperationType::default() as i32,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt64::new(
                        "download-size",
                        "",
                        "",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt64::new(
                        "installed-size",
                        "",
                        "",
                        0,
                        u64::MAX,
                        0,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "package" => self.obj().package().to_value(),
                "operation-type" => self.obj().operation_type().to_value(),
                "download-size" => self.obj().download_size().to_value(),
                "installed-size" => self.obj().installed_size().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkDryRunRuntime(ObjectSubclass<imp::SkDryRunRuntime>);
}

impl SkDryRunRuntime {
    pub fn new(data: DryRunRuntime) -> Self {
        let runtime: Self = glib::Object::new(&[]);
        let imp = runtime.imp();

        let package = SkPackage::new(&data.package);
        imp.package.set(package).unwrap();

        imp.data.set(data).unwrap();

        runtime
    }

    pub fn data(&self) -> DryRunRuntime {
        self.imp().data.get().unwrap().clone()
    }

    pub fn package(&self) -> SkPackage {
        self.imp().package.get().unwrap().clone()
    }

    pub fn operation_type(&self) -> SkFlatpakOperationType {
        self.data().operation_type.into()
    }

    pub fn download_size(&self) -> u64 {
        self.data().download_size
    }

    pub fn installed_size(&self) -> u64 {
        self.data().installed_size
    }

    pub fn appstream(&self) -> SkPackageAppstream {
        SkPackageAppstream::from_dry_run_runtime(self)
    }

    pub fn metadata(&self) -> KeyFile {
        let keyfile = KeyFile::new();
        keyfile
            .load_from_data(&self.data().metadata, KeyFileFlags::NONE)
            .unwrap();
        keyfile
    }

    pub fn old_metadata(&self) -> Option<KeyFile> {
        if let Some(metadata) = self.data().old_metadata.into() {
            let metadata: String = metadata;
            let keyfile = KeyFile::new();
            keyfile
                .load_from_data(&metadata, KeyFileFlags::NONE)
                .unwrap();
            Some(keyfile)
        } else {
            None
        }
    }
}
