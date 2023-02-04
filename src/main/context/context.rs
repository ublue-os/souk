// Souk - context.rs
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

use glib::{ParamFlags, ParamSpec, ParamSpecObject, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::context::{SkContextDetail, SkContextDetailGroup, SkContextDetailGroupModel};
use crate::main::flatpak::permissions::types::{SkFilesystemPermission, SkServicePermission};
use crate::main::flatpak::permissions::{PermissionDetails, SkAppPermissions, SkPermissionSummary};
use crate::main::i18n::i18n;

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

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "summary" => self.obj().summary().to_value(),
                "details" => self.obj().details().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
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
        glib::Object::new(&[("summary", &summary), ("details", &details)])
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
        let summary = summary.as_context_detail();

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
