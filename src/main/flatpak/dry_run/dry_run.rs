// Souk - dry_run.rs
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
    KeyFile, KeyFileFlags, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject,
    ParamSpecUInt64, ToValue,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::SkDryRunRuntimeModel;
use crate::main::context::SkContext;
use crate::main::flatpak::installation::{SkRemote, SkRemoteModel};
use crate::main::flatpak::package::{SkPackage, SkPackageAppstream};
use crate::main::flatpak::permissions::SkAppPermissions;
use crate::main::flatpak::SkFlatpakOperationType;
use crate::shared::flatpak::dry_run::DryRun;
use crate::shared::flatpak::info::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRun {
        pub data: OnceCell<DryRun>,

        pub package: OnceCell<SkPackage>,
        pub runtimes: SkDryRunRuntimeModel,
        pub added_remotes: SkRemoteModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRun {
        const NAME: &'static str = "SkDryRun";
        type Type = super::SkDryRun;
    }

    impl ObjectImpl for SkDryRun {
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
                    ParamSpecObject::new(
                        "runtimes",
                        "",
                        "",
                        SkDryRunRuntimeModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "added-remotes",
                        "",
                        "",
                        SkRemoteModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecBoolean::new("has-update-source", "", "", false, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "is-replacing-remote",
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
                "package" => self.obj().package().to_value(),
                "operation-type" => self.obj().operation_type().to_value(),
                "download-size" => self.obj().download_size().to_value(),
                "installed-size" => self.obj().installed_size().to_value(),
                "runtimes" => self.obj().runtimes().to_value(),
                "added-remotes" => self.obj().added_remotes().to_value(),
                "has-update-source" => self.obj().has_update_source().to_value(),
                "is-replacing-remote" => self.obj().is_replacing_remote().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkDryRun(ObjectSubclass<imp::SkDryRun>);
}

impl SkDryRun {
    pub fn new(data: DryRun) -> Self {
        let dry_run: Self = glib::Object::new(&[]);
        let imp = dry_run.imp();

        let package = SkPackage::new(&data.package);
        imp.package.set(package).unwrap();

        imp.runtimes.set_runtimes(data.runtimes.clone());
        imp.added_remotes.set_remotes(data.added_remotes.clone());

        imp.data.set(data).unwrap();

        dry_run
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

    pub fn runtimes(&self) -> SkDryRunRuntimeModel {
        self.imp().runtimes.clone()
    }

    pub fn added_remotes(&self) -> SkRemoteModel {
        self.imp().added_remotes.clone()
    }

    pub fn has_update_source(&self) -> bool {
        self.data().has_update_source
    }

    pub fn is_replacing_remote(&self) -> Option<SkRemote> {
        let remote_info: Option<RemoteInfo> = self.data().is_replacing_remote.into();
        remote_info.map(|remote_info| SkRemote::new(&remote_info))
    }

    pub fn appstream(&self) -> SkPackageAppstream {
        SkPackageAppstream::from_dry_run(self)
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

    // TODO: Rework context info so it makes use of new SkDryRun objects etc.
    pub fn permissions(&self) -> SkAppPermissions {
        SkAppPermissions::from_metadata(&self.metadata())
    }

    pub fn old_permissions(&self) -> Option<SkAppPermissions> {
        self.old_metadata()
            .map(|m| SkAppPermissions::from_metadata(&m))
    }

    pub fn permissions_context(&self) -> SkContext {
        SkContext::permissions(&self.permissions())
    }

    pub fn download_size_context(&self) -> SkContext {
        SkContext::download_size(self)
    }

    pub fn installed_size_context(&self) -> SkContext {
        SkContext::installed_size(self)
    }

    pub fn data(&self) -> DryRun {
        self.imp().data.get().unwrap().clone()
    }
}
