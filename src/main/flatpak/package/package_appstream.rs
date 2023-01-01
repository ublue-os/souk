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

use crate::main::i18n::{i18n, i18n_f};
use crate::main::SkApplication;
use crate::shared::dry_run::DryRunResult;
use crate::shared::info::PackageInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkPackageAppstream {
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
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon" => obj.icon().to_value(),
                "name" => obj.name().to_value(),
                "developer-name" => obj.developer_name().to_value(),
                "version" => obj.version().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkPackageAppstream(ObjectSubclass<imp::SkPackageAppstream>);
}

impl SkPackageAppstream {
    pub fn from_dry_run(dry_run: &DryRunResult) -> Self {
        let appstream: Self = glib::Object::new(&[]).unwrap();
        let imp = appstream.imp();

        // Appstream Component
        let text = dry_run
            .appstream_component
            .as_ref()
            .cloned()
            .unwrap_or_default();
        let c = serde_json::from_str(&text).unwrap_or(Self::fallback_component(&dry_run.package));
        imp.component.set(c).unwrap();

        // Icon
        let icon = dry_run.icon.clone();
        let bytes = Bytes::from_owned(icon);
        let icon: Paintable = if let Ok(texture) = gdk::Texture::from_bytes(&bytes) {
            texture.upcast()
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
        let value = &self.imp().component.get().unwrap().name;
        self.translated_value(value)
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
    pub fn version_text(&self) -> String {
        let mut releases = self.imp().component.get().unwrap().releases.clone();
        releases.sort_by(|r1, r2| r1.version.cmp(&r2.version));
        if let Some(release) = releases.get(0) {
            i18n_f("Version {}", &[&release.version.clone()])
        } else {
            i18n("Unknown Version")
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
        value
            .get_for_locale(&self.locale())
            .unwrap_or(value.get_default().unwrap())
            .to_string()
    }

    fn locale(&self) -> String {
        let f_inst = Installation::from(
            &SkApplication::default()
                .worker()
                .installations()
                .preferred()
                .info(),
        );

        f_inst
            .default_languages()
            .unwrap()
            .first()
            .unwrap()
            .to_string()
    }
}
