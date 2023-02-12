// Souk - dry_run_package.rs
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
use url::Url;

use crate::main::context::{SkContext, SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::package::{SkPackage, SkPackageAppstream, SkPackageImpl};
use crate::main::flatpak::permissions::SkAppPermissions;
use crate::main::flatpak::SkFlatpakOperationType;
use crate::main::i18n::i18n_f;
use crate::shared::flatpak::dry_run::DryRunPackage;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRunPackage {
        pub data: OnceCell<DryRunPackage>,
        pub package: OnceCell<SkPackage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRunPackage {
        const NAME: &'static str = "SkDryRunPackage";
        type Type = super::SkDryRunPackage;
        type ParentType = SkPackage;
    }

    impl ObjectImpl for SkDryRunPackage {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
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
                        "appstream",
                        "",
                        "",
                        SkPackageAppstream::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "operation-type" => self.obj().operation_type().to_value(),
                "download-size" => self.obj().download_size().to_value(),
                "installed-size" => self.obj().installed_size().to_value(),
                "appstream" => self.obj().appstream().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl SkPackageImpl for SkDryRunPackage {}
}

glib::wrapper! {
    pub struct SkDryRunPackage(ObjectSubclass<imp::SkDryRunPackage>) @extends SkPackage;
}

impl SkDryRunPackage {
    pub fn new(data: DryRunPackage) -> Self {
        let runtime: Self = glib::Object::builder()
            .property("info", &data.package)
            .build();

        let imp = runtime.imp();
        imp.data.set(data).unwrap();

        runtime
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
        SkPackageAppstream::new(
            self.data().appstream_component.into(),
            self.data().icon.into(),
            self.upcast_ref(),
        )
    }

    pub fn size_context_detail(&self, download_size: bool) -> SkContextDetail {
        // Optional extra data line
        let mut extra_data_line = String::new();
        if let Some(url) = self.extra_data_source() {
            let domain = url.domain().unwrap();

            let msg = if download_size {
                i18n_f(
                    "Downloads additional data from an external source ({})",
                    &[domain],
                )
            } else {
                i18n_f(
                    "Requires additional data from an external source ({}) with unknown disk usage",
                    &[domain],
                )
            };

            extra_data_line =
                format!("\n<small><b><span foreground=\"orange\" baseline_shift=\"-18pt\">{msg}</span></b></small>");
        }

        // Version line
        let mut version_line = self.appstream().version_text(true);
        if self.operation_type() != SkFlatpakOperationType::Install
            && self.operation_type() != SkFlatpakOperationType::InstallBundle
        {
            version_line = i18n_f(
                "{} – {}",
                &[&version_line, &self.operation_type().to_string()],
            );
        }

        let subtitle = format!(
            "{}{}\n<small><span baseline_shift=\"-18pt\">{}</span></small>",
            self.appstream().summary(),
            extra_data_line,
            version_line
        );

        let size = if download_size {
            self.download_size()
        } else {
            self.installed_size()
        };

        if self.extra_data_source().is_some() && !download_size {
            SkContextDetail::new(
                SkContextDetailKind::Size,
                "???",
                SkContextDetailLevel::Neutral,
                &self.appstream().name(),
                &subtitle,
            )
        } else {
            SkContextDetail::new_neutral_size(size, &self.appstream().name(), &subtitle)
        }
    }

    pub fn extra_data_source(&self) -> Option<Url> {
        let keyfile = self.metadata();
        if keyfile.has_group("Extra Data") {
            let uri = keyfile.string("Extra Data", "uri").unwrap();
            return Some(Url::parse(&uri).unwrap());
        }

        None
    }

    // TODO: Include old permissions as well
    pub fn permissions_context(&self) -> SkContext {
        SkContext::permissions(&self.permissions())
    }

    // TODO: Rework context info so it makes use of new SkDryRun objects etc.
    pub fn permissions(&self) -> SkAppPermissions {
        // TODO: Make this a gobject property of SkDryRun
        SkAppPermissions::from_metadata(&self.metadata())
    }

    pub fn old_permissions(&self) -> Option<SkAppPermissions> {
        self.old_metadata()
            .map(|m| SkAppPermissions::from_metadata(&m))
    }

    fn metadata(&self) -> KeyFile {
        let keyfile = KeyFile::new();
        keyfile
            .load_from_data(&self.data().metadata, KeyFileFlags::NONE)
            .unwrap();
        keyfile
    }

    fn old_metadata(&self) -> Option<KeyFile> {
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

    fn data(&self) -> DryRunPackage {
        self.imp().data.get().unwrap().clone()
    }
}
