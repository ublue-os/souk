// Souk - package_appstream.rs
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

use appstream::builders::ComponentBuilder;
use appstream::{AppId, Component, TranslatableString};
use flatpak::prelude::*;
use flatpak::{Installation, Ref};
use glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, ToValue};
use gtk::gdk::Paintable;
use gtk::glib;
use gtk::glib::Bytes;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::package::{SkPackage, SkPackageKind, SkPackageSubrefKind};
use crate::main::i18n::{i18n, i18n_f};
use crate::main::SkApplication;
use crate::shared::flatpak::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackageAppstream {
        pub package: OnceCell<SkPackage>,

        pub component: OnceCell<Component>,
        pub icon: OnceCell<Paintable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackageAppstream {
        const NAME: &'static str = "SkPackageAppstream";
        type Type = super::SkPackageAppstream;
    }

    impl ObjectImpl for SkPackageAppstream {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "icon",
                        "",
                        "",
                        Paintable::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("developer-name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("version", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("summary", "", "", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon" => self.obj().icon().to_value(),
                "name" => self.obj().name().to_value(),
                "developer-name" => self.obj().developer_name().to_value(),
                "version" => self.obj().version().to_value(),
                "summary" => self.obj().summary().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkPackageAppstream(ObjectSubclass<imp::SkPackageAppstream>);
}

impl SkPackageAppstream {
    pub fn new(
        appstream_string: Option<String>,
        icon: Option<Vec<u8>>,
        package: &SkPackage,
    ) -> Self {
        let appstream: Self = glib::Object::new();

        let imp = appstream.imp();
        imp.package.set(package.clone()).unwrap();

        // Appstream Component
        let text = appstream_string.unwrap_or_default();

        let fallback = package.info();
        let c = serde_json::from_str(&text).unwrap_or_else(|_| Self::fallback_component(&fallback));
        imp.component.set(c).unwrap();

        // Icon
        let icon = if let Some(icon) = icon {
            let bytes = Bytes::from_owned(icon);
            if let Ok(texture) = gdk::Texture::from_bytes(&bytes) {
                texture.upcast()
            } else {
                Self::fallback_icon().upcast()
            }
        } else {
            Self::fallback_icon().upcast()
        };
        imp.icon.set(icon).unwrap();

        appstream
    }

    pub fn icon(&self) -> Paintable {
        self.imp().icon.get().unwrap().clone()
    }

    pub fn name(&self) -> String {
        let imp = self.imp();
        let component = imp.component.get().unwrap();
        let package = imp.package.get().unwrap();

        let mut name = self.translated_value(&component.name);
        match package.subref_kind() {
            SkPackageSubrefKind::Locale => name = i18n_f("{} (Translations)", &[&name]),
            SkPackageSubrefKind::Debug => name = i18n_f("{} (Debug)", &[&name]),
            SkPackageSubrefKind::Sources => name = i18n_f("{} (Sources)", &[&name]),
            SkPackageSubrefKind::None => (),
        }

        name
    }

    pub fn developer_name(&self) -> String {
        if let Some(value) = &self.imp().component.get().unwrap().developer_name {
            self.translated_value(value)
        } else {
            i18n("Unknown Developer")
        }
    }

    /// Returns just the version as number, eg. "3.1"
    pub fn version(&self) -> String {
        let mut releases = self.imp().component.get().unwrap().releases.clone();
        releases.sort_by(|r1, r2| r1.version.cmp(&r2.version));
        if let Some(release) = releases.get(0) {
            release.version.clone()
        } else {
            "–".into()
        }
    }

    /// Returns the version as user friendly text, eg. "Version 3.1" or "Unknown
    /// Version"
    pub fn version_text(&self, include_branch: bool) -> String {
        let mut releases = self.imp().component.get().unwrap().releases.clone();
        releases.sort_by(|r1, r2| r1.version.cmp(&r2.version));

        let branch = self.imp().package.get().unwrap().branch();
        let version = if let Some(release) = releases.get(0) {
            if include_branch {
                format!("{} ({})", release.version.clone(), branch)
            } else {
                release.version.clone()
            }
        } else {
            branch
        };
        i18n_f("Version {}", &[&version])
    }

    pub fn summary(&self) -> String {
        let imp = self.imp();
        let component = imp.component.get().unwrap();
        let package = imp.package.get().unwrap();

        match package.subref_kind() {
            SkPackageSubrefKind::Locale => return i18n("Translations for various languages"),
            SkPackageSubrefKind::Debug => return i18n("Development and diagnostics data"),
            SkPackageSubrefKind::Sources => return i18n("Source code"),
            SkPackageSubrefKind::None => (),
        }

        if let Some(value) = &component.summary {
            self.translated_value(value)
        } else if package.kind() == SkPackageKind::Runtime {
            i18n("A Flatpak Runtime")
        } else {
            i18n("A Flatpak Application")
        }
    }

    fn fallback_component(package: &PackageInfo) -> Component {
        let ref_ = Ref::parse(&package.ref_).unwrap();
        let app_id = ref_.name().unwrap().to_string();
        let name = TranslatableString::with_default(&app_id);

        ComponentBuilder::default()
            .id(AppId(app_id))
            .name(name)
            .build()
    }

    fn fallback_icon() -> Paintable {
        gdk::Texture::from_resource(
            "/de/haeckerfelix/Souk/icons/128x128/emblems/system-component.svg",
        )
        .upcast()
    }

    fn translated_value(&self, value: &TranslatableString) -> String {
        let locale = self.locale().unwrap_or("C".to_string());
        value
            .get_for_locale(&locale)
            .unwrap_or_else(|| value.get_default().unwrap())
            .to_string()
    }

    fn locale(&self) -> Option<String> {
        let f_inst = Installation::from(
            &SkApplication::default()
                .worker()
                .installations()
                .preferred()
                .info(),
        );

        let locale = f_inst.default_languages().ok()?.first()?.to_string();
        Some(locale)
    }
}
