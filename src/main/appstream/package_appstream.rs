// Souk - package_appstream.rs
// Copyright (C) 2022-2024  Felix Häcker <haeckerfelix@gnome.org>
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
use glib::{ParamSpec, Properties};
use gtk::gdk::Paintable;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

use crate::main::flatpak::package::{SkPackage, SkPackageKind, SkPackageSubrefKind};
use crate::main::i18n::{i18n, i18n_f};
use crate::main::SkApplication;
use crate::shared::flatpak::info::PackageInfo;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "Component")]
pub struct BoxedComponent(Component);

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkPackageAppstream)]
    pub struct SkPackageAppstream {
        #[property(get, set, construct_only)]
        pub package: OnceCell<SkPackage>,
        #[property(get, set, construct_only)]
        icon: OnceCell<Paintable>,
        #[property(get, set, construct_only)]
        #[property(name = "name", get = Self::name, type = String)]
        #[property(name = "developer-name", get = Self::developer_name, type = String)]
        #[property(name = "version", get = Self::version, type = String)]
        #[property(name = "summary", get = Self::summary, type = String)]
        component: OnceCell<BoxedComponent>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkPackageAppstream {
        const NAME: &'static str = "SkPackageAppstream";
        type Type = super::SkPackageAppstream;
    }

    impl ObjectImpl for SkPackageAppstream {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
    }

    impl SkPackageAppstream {
        fn name(&self) -> String {
            let mut name = self.translated_value(&self.obj().component().0.name);
            match self.obj().package().subref_kind() {
                SkPackageSubrefKind::Locale => name = i18n_f("{} (Translations)", &[&name]),
                SkPackageSubrefKind::Debug => name = i18n_f("{} (Debug)", &[&name]),
                SkPackageSubrefKind::Sources => name = i18n_f("{} (Sources)", &[&name]),
                SkPackageSubrefKind::None => (),
            }

            name
        }

        fn developer_name(&self) -> String {
            if let Some(value) = &self.obj().component().0.developer_name {
                self.translated_value(value)
            } else {
                i18n("Unknown Developer")
            }
        }

        /// Returns just the version as number, eg. "3.1"
        fn version(&self) -> String {
            let mut releases = self.obj().component().0.releases;
            releases.sort_by(|r1, r2| r2.version.cmp(&r1.version));
            if let Some(release) = releases.first() {
                release.version.clone()
            } else {
                "–".into()
            }
        }

        fn summary(&self) -> String {
            match self.obj().package().subref_kind() {
                SkPackageSubrefKind::Locale => return i18n("Translations for various languages"),
                SkPackageSubrefKind::Debug => return i18n("Development and diagnostics data"),
                SkPackageSubrefKind::Sources => return i18n("Source code"),
                SkPackageSubrefKind::None => (),
            }

            if let Some(value) = self.obj().component().0.summary {
                self.translated_value(&value)
            } else if self.obj().package().kind() == SkPackageKind::Runtime {
                i18n("A Flatpak Runtime")
            } else {
                i18n("A Flatpak Application")
            }
        }

        fn translated_value(&self, value: &TranslatableString) -> String {
            let locale = self.locale().unwrap_or("C".to_string());
            let fallback = "–".to_string();
            value
                .get_for_locale(&locale)
                .unwrap_or_else(|| value.get_default().unwrap_or(&fallback))
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
}

glib::wrapper! {
    pub struct SkPackageAppstream(ObjectSubclass<imp::SkPackageAppstream>);
}

impl SkPackageAppstream {
    pub fn new(package: &SkPackage, icon: &Paintable, component: Component) -> Self {
        glib::Object::builder()
            .property("package", package)
            .property("icon", icon)
            .property("component", BoxedComponent(component))
            .build()
    }

    pub fn fallback_component(package: &PackageInfo) -> Component {
        let ref_ = Ref::parse(&package.ref_).unwrap();
        let app_id = ref_.name().unwrap().to_string();
        let name = TranslatableString::with_default(&app_id);

        ComponentBuilder::default()
            .id(AppId(app_id))
            .name(name)
            .build()
    }

    pub fn fallback_icon() -> Paintable {
        gdk::Texture::from_resource(
            "/de/haeckerfelix/Souk/icons/128x128/emblems/system-component.svg",
        )
        .upcast()
    }

    /// Returns the version as user friendly text, eg. "Version 3.1" or "Unknown
    /// Version"
    pub fn version_text(&self, include_branch: bool) -> String {
        let mut releases = self.component().0.releases;
        releases.sort_by(|r1, r2| r2.version.cmp(&r1.version));

        let branch = self.imp().package.get().unwrap().branch();
        let version = if let Some(release) = releases.first() {
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
}
