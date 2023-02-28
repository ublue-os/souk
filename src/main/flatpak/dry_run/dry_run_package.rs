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

use glib::{Bytes, KeyFile, KeyFileFlags, ParamSpec, Properties};
use gtk::gdk::Paintable;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use url::Url;

use crate::main::context::{SkContext, SkContextDetail, SkContextDetailKind, SkContextDetailLevel};
use crate::main::flatpak::package::{SkPackage, SkPackageAppstream, SkPackageExt, SkPackageImpl};
use crate::main::flatpak::permissions::SkAppPermissions;
use crate::main::flatpak::SkFlatpakOperationKind;
use crate::main::i18n::i18n_f;
use crate::shared::flatpak::dry_run::DryRunPackage;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkDryRunPackage)]
    pub struct SkDryRunPackage {
        #[property(get)]
        package: OnceCell<SkPackage>,
        #[property(name = "appstream", get)]
        appstream: OnceCell<SkPackageAppstream>,
        #[property(get, set, construct_only)]
        #[property(name = "operation-kind", get = Self::operation_kind, type = SkFlatpakOperationKind, builder(SkFlatpakOperationKind::None))]
        #[property(name = "download-size", get, type = u64, member = download_size)]
        #[property(name = "installed-size", get, type = u64, member = installed_size)]
        data: OnceCell<DryRunPackage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkDryRunPackage {
        const NAME: &'static str = "SkDryRunPackage";
        type Type = super::SkDryRunPackage;
        type ParentType = SkPackage;
    }

    impl ObjectImpl for SkDryRunPackage {
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
            let data = self.obj().data();

            // Set appstream
            let icon: Paintable = if let Some(icon) = data.icon.as_ref() {
                let bytes = Bytes::from_owned(icon.clone());
                if let Ok(texture) = gdk::Texture::from_bytes(&bytes) {
                    texture.upcast()
                } else {
                    SkPackageAppstream::fallback_icon().upcast()
                }
            } else {
                SkPackageAppstream::fallback_icon().upcast()
            };

            let package_info = self.obj().info();
            let component = if let Some(json) = data.appstream_component.as_ref() {
                serde_json::from_str(json)
                    .unwrap_or_else(|_| SkPackageAppstream::fallback_component(&package_info))
            } else {
                SkPackageAppstream::fallback_component(&package_info)
            };

            let package: SkPackage = self.obj().clone().upcast();
            let appstream = SkPackageAppstream::new(&package, &icon, component);
            self.appstream.set(appstream).unwrap();
        }
    }

    impl SkPackageImpl for SkDryRunPackage {}

    impl SkDryRunPackage {
        fn operation_kind(&self) -> SkFlatpakOperationKind {
            self.obj().data().operation_kind.into()
        }

        pub fn metadata(&self) -> KeyFile {
            let keyfile = KeyFile::new();
            keyfile
                .load_from_data(&self.obj().data().metadata, KeyFileFlags::NONE)
                .unwrap();
            keyfile
        }

        pub fn old_metadata(&self) -> Option<KeyFile> {
            if let Some(metadata) = self.obj().data().old_metadata.into() {
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
}

glib::wrapper! {
    pub struct SkDryRunPackage(ObjectSubclass<imp::SkDryRunPackage>) @extends SkPackage;
}

impl SkDryRunPackage {
    pub fn new(data: DryRunPackage) -> Self {
        glib::Object::builder()
            .property("info", &data.info)
            .property("data", &data)
            .build()
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
                format!("\n<small><b><span foreground=\"#ff7800\" baseline_shift=\"-18pt\">{msg}</span></b></small>");
        }

        // Version line
        let mut version_line = self.appstream().version_text(true);
        if self.operation_kind() != SkFlatpakOperationKind::Install
            && self.operation_kind() != SkFlatpakOperationKind::InstallBundle
        {
            version_line = i18n_f(
                "{} – <b><span foreground=\"#3584e4\">{}</span></b>",
                &[&version_line, &self.operation_kind().to_string()],
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
        let keyfile = self.imp().metadata();
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
        SkAppPermissions::from_metadata(&self.imp().metadata())
    }

    pub fn old_permissions(&self) -> Option<SkAppPermissions> {
        self.imp()
            .old_metadata()
            .map(|m| SkAppPermissions::from_metadata(&m))
    }
}
