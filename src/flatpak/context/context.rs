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
    SkContextDetail, SkContextDetailGroup, SkContextDetailGroupModel, SkContextDetailLevel,
    SkContextDetailType,
};
use crate::flatpak::permissions::types::{SkFilesystemPermission, SkServicePermission};
use crate::flatpak::permissions::{PermissionDetails, SkAppPermissions, SkPermissionSummary};
use crate::flatpak::utils;
use crate::i18n::{i18n, i18n_f};
use crate::worker::DryRunResult;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkContext {
        pub summary: OnceCell<SkContextDetail>,
        pub details: OnceCell<SkContextDetailGroupModel>,
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
                        SkContextDetailGroupModel::static_type(),
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
    pub fn new(summary: &SkContextDetail, details: &SkContextDetailGroupModel) -> Self {
        glib::Object::new(&[("summary", &summary), ("details", &details)]).unwrap()
    }

    pub fn permissions(permissions: &SkAppPermissions) -> Self {
        let mut groups = Vec::new();
        let mut summary = SkPermissionSummary::empty();

        // General
        let mut general_details = Vec::new();
        general_details.append(&mut permissions.subsystems().context_details());
        general_details.append(&mut permissions.devices().context_details());
        general_details.append(&mut permissions.sockets().context_details());
        summary |= permissions.subsystems().summary();
        summary |= permissions.devices().summary();
        summary |= permissions.sockets().summary();

        let description = i18n("The isolated environment does not protect against malicious applications. Applications can request additional permissions at runtime. However, these must be explicitly confirmed.");
        let group = SkContextDetailGroup::new(&general_details, None, Some(&description));
        groups.push(group);

        // Filesystems
        let mut filesystem_details = Vec::new();
        for value in permissions.filesystems().snapshot() {
            let value: SkFilesystemPermission = value.downcast().unwrap();
            filesystem_details.push(value.context_details()[0].clone());
            summary |= value.summary();
        }
        if permissions.filesystems().n_items() == 0 {
            filesystem_details.push(SkFilesystemPermission::no_permission_context());
        }

        let title = i18n("Filesystem Permissions");
        let group = SkContextDetailGroup::new(&filesystem_details, Some(&title), None);
        groups.push(group);

        // Services
        let mut service_details = Vec::new();
        for value in permissions.services().snapshot() {
            let value: SkServicePermission = value.downcast().unwrap();
            service_details.push(value.context_details()[0].clone());
            summary |= value.summary();
        }
        if permissions.services().n_items() == 0 {
            service_details.push(SkServicePermission::no_permission_context());
        }

        let title = i18n("Service Permissions");
        let group = SkContextDetailGroup::new(&service_details, Some(&title), None);
        groups.push(group);

        // Summary
        let summary = summary.to_context_detail();

        let groups = SkContextDetailGroupModel::new(&groups);
        Self::new(&summary, &groups)
    }

    pub fn download_size(dry_run: &DryRunResult) -> Self {
        Self::size_context(dry_run, true)
    }

    pub fn installed_size(dry_run: &DryRunResult) -> Self {
        Self::size_context(dry_run, false)
    }

    fn size_context(dry_run: &DryRunResult, download_size: bool) -> Self {
        let mut groups = Vec::new();
        let mut runtime_size: u64 = 0;

        // Sort by size
        let mut runtimes = dry_run.runtimes.clone();
        if download_size {
            runtimes.sort_by(|a, b| b.download_size.cmp(&a.download_size));
        } else {
            runtimes.sort_by(|a, b| b.installed_size.cmp(&a.installed_size));
        }

        // The package itelf
        let mut package_details = Vec::new();

        let size = if download_size {
            dry_run.download_size
        } else {
            dry_run.installed_size
        };
        let package_size = size;

        let package_ref = Ref::parse(&dry_run.ref_).unwrap();
        let package_ref_name = package_ref.name().unwrap().to_string();
        let package_ref_branch = package_ref.branch().unwrap().to_string();

        let title = i18n("Application Data");
        let detail = if dry_run.has_extra_data() && !download_size {
            let subtitle = i18n_f("{} ({}) – Requires additional extra data from an external source with unknown size", &[&package_ref_name, &package_ref_branch]);
            SkContextDetail::new_neutral_text("  ???  ", &title, &subtitle)
        } else {
            let subtitle = format!("{} ({})", package_ref_name, package_ref_branch);
            SkContextDetail::new_neutral_size(size, &title, &subtitle)
        };
        package_details.push(detail);

        // Runtimes
        let mut runtime_details = Vec::new();
        for runtime in &runtimes {
            let ref_ = Ref::parse(&runtime.ref_).unwrap();
            let ref_name = ref_.name().unwrap().to_string();
            let ref_branch = ref_.branch().unwrap().to_string();

            let mut title = utils::runtime_ref_to_display_name(&ref_name);
            if runtime.operation_type == "update" {
                title = i18n_f("{} (Update)", &[&title]);
            }
            let subtitle = format!("{} ({})", ref_name, ref_branch);

            let size = if download_size {
                runtime.download_size
            } else {
                runtime.installed_size
            };
            runtime_size += size;

            let detail = SkContextDetail::new_neutral_size(size, &title, &subtitle);
            if ref_name.contains(&package_ref_name) {
                package_details.push(detail);
            } else {
                runtime_details.push(detail);
            }
        }

        let description = i18n("The storage sizes are only maximum values. Actual usage will most likely be significantly lower due to deduplication of data.");
        let group = SkContextDetailGroup::new(&package_details, None, Some(&description));
        groups.push(group);

        if !dry_run.runtimes.is_empty() {
            let description = i18n("These components are shared with other applications, and only need to be downloaded once.");
            let group = SkContextDetailGroup::new(&runtime_details, None, Some(&description));
            groups.push(group);
        }

        // Summary
        let total_size = package_size + runtime_size;
        let total_size_str = glib::format_size(total_size);
        let runtime_size_str = glib::format_size(runtime_size);
        let summary = if download_size {
            let title = if total_size == 0 {
                i18n("No download required")
            } else {
                i18n_f("Up to {} to download", &[&total_size_str])
            };

            let descr = if runtime_size == 0 {
                i18n("No additional system packages needed")
            } else {
                i18n_f(
                    "Needs {} of additional system packages",
                    &[&runtime_size_str],
                )
            };

            SkContextDetail::new(
                SkContextDetailType::Icon,
                "folder-download-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        } else {
            let title = if dry_run.has_extra_data() {
                i18n("Unknown storage size")
            } else {
                i18n_f("Up to {} storage required", &[&total_size_str])
            };

            let descr = if runtime_size == 0 {
                i18n("Requires no additional space for system packages")
            } else {
                i18n_f(
                    "Requires {} for shared system packages",
                    &[&runtime_size_str],
                )
            };
            SkContextDetail::new(
                SkContextDetailType::Icon,
                "drive-harddisk-system-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        };

        let groups = SkContextDetailGroupModel::new(&groups);
        Self::new(&summary, &groups)
    }

    pub fn summary(&self) -> SkContextDetail {
        self.imp().summary.get().unwrap().clone()
    }

    pub fn details(&self) -> SkContextDetailGroupModel {
        self.imp().details.get().unwrap().clone()
    }
}
