// Souk - context.rs
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
use glib::{ParamFlags, ParamSpec, ParamSpecObject, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::flatpak::context::{
    SkContextDetail, SkContextDetailLevel, SkContextDetailModel, SkContextDetailType,
};
use crate::flatpak::utils;
use crate::i18n::{i18n, i18n_f};
use crate::worker::TransactionDryRun;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContext {
        pub summary: OnceCell<SkContextDetail>,
        pub details: OnceCell<SkContextDetailModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkContext {
        const NAME: &'static str = "SkContext";
        type ParentType = glib::Object;
        type Type = super::SkContext;
    }

    impl ObjectImpl for SkContext {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "summary",
                        "",
                        "",
                        SkContextDetail::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecObject::new(
                        "details",
                        "",
                        "",
                        SkContextDetailModel::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "summary" => obj.summary().to_value(),
                "details" => obj.details().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "summary" => self.summary.set(value.get().unwrap()).unwrap(),
                "details" => self.details.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkContext(ObjectSubclass<imp::SkContext>);
}

impl SkContext {
    pub fn new(summary: &SkContextDetail, details: &SkContextDetailModel) -> Self {
        glib::Object::new(&[("summary", &summary), ("details", &details)]).unwrap()
    }

    pub fn download_size(dry_run: &TransactionDryRun) -> Self {
        Self::size_context(dry_run, true)
    }

    pub fn installed_size(dry_run: &TransactionDryRun) -> Self {
        Self::size_context(dry_run, false)
    }

    fn size_context(dry_run: &TransactionDryRun, download_size: bool) -> Self {
        let mut details = Vec::new();
        let mut runtime_size: u64 = 0;

        // Sort by size
        let mut runtimes = dry_run.runtimes.clone();
        if download_size {
            runtimes.sort_by(|a, b| b.download_size.cmp(&a.download_size));
        } else {
            runtimes.sort_by(|a, b| b.installed_size.cmp(&a.installed_size));
        }

        // The package itelf
        let title = i18n("Application Data");
        let description = i18n("The application itself");
        let size = if download_size {
            dry_run.download_size
        } else {
            dry_run.installed_size
        };
        let detail = SkContextDetail::new_neutral_size(size, &title, &description);
        let package_size = size;
        details.push(detail);

        // Runtimes
        for runtime in &runtimes {
            let ref_ = Ref::parse(&runtime.ref_).unwrap();
            let ref_name = ref_.name().unwrap().to_string();

            let title = utils::runtime_ref_to_display_name(&ref_name);
            let size = if download_size {
                runtime.download_size
            } else {
                runtime.installed_size
            };
            runtime_size += size;

            let detail = SkContextDetail::new_neutral_size(size, &title, &ref_name);
            details.push(detail);
        }

        // Summary
        let total_size = package_size + runtime_size;
        let total_size = glib::format_size(total_size);
        let runtime_size = glib::format_size(runtime_size);
        let summary = if download_size {
            let title = i18n_f("Up to {} to download", &[&total_size]);
            let descr = i18n_f("Needs {} of additional system packages", &[&runtime_size]);
            SkContextDetail::new(
                SkContextDetailType::Icon,
                "folder-download-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        } else {
            let title = i18n_f("Up to {} storage required", &[&total_size]);
            let descr = i18n_f("Requires {} for shared system packages", &[&runtime_size]);
            SkContextDetail::new(
                SkContextDetailType::Icon,
                "drive-harddisk-system-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        };

        let details = SkContextDetailModel::new(&details);
        Self::new(&summary, &details)
    }

    pub fn summary(&self) -> SkContextDetail {
        self.imp().summary.get().unwrap().clone()
    }

    pub fn details(&self) -> SkContextDetailModel {
        self.imp().details.get().unwrap().clone()
    }
}
