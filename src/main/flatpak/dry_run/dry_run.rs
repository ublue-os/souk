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

use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::{SkDryRunPackage, SkDryRunPackageModel};
use crate::main::context::{
    SkContext, SkContextDetail, SkContextDetailGroup, SkContextDetailGroupModel,
    SkContextDetailKind, SkContextDetailLevel,
};
use crate::main::flatpak::installation::{SkRemote, SkRemoteModel};
use crate::main::flatpak::package::SkPackageExt;
use crate::main::i18n::{i18n, i18n_f};
use crate::shared::flatpak::dry_run::DryRun;
use crate::shared::flatpak::info::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkDryRun {
        pub data: OnceCell<DryRun>,

        pub package: OnceCell<SkDryRunPackage>,
        pub runtimes: SkDryRunPackageModel,
        pub remotes: SkRemoteModel,
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
                        SkDryRunPackage::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "runtimes",
                        "",
                        "",
                        SkDryRunPackageModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "remotes",
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
                "runtimes" => self.obj().runtimes().to_value(),
                "remotes" => self.obj().remotes().to_value(),
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
        let dry_run: Self = glib::Object::new();
        let imp = dry_run.imp();

        let package = SkDryRunPackage::new(data.package.clone());
        imp.package.set(package).unwrap();

        imp.runtimes.set_packages(data.runtimes.clone());
        imp.remotes.set_remotes(data.remotes.clone());

        imp.data.set(data).unwrap();

        dry_run
    }

    pub fn package(&self) -> SkDryRunPackage {
        self.imp().package.get().unwrap().clone()
    }

    pub fn runtimes(&self) -> SkDryRunPackageModel {
        self.imp().runtimes.clone()
    }

    pub fn remotes(&self) -> SkRemoteModel {
        self.imp().remotes.clone()
    }

    pub fn has_update_source(&self) -> bool {
        self.data().has_update_source
    }

    pub fn is_replacing_remote(&self) -> Option<SkRemote> {
        let remote_info: Option<RemoteInfo> = self.data().is_replacing_remote.into();
        remote_info.map(|remote_info| SkRemote::new(&remote_info))
    }

    pub fn download_size_context(&self) -> SkContext {
        self.size_context(false)
    }

    pub fn installed_size_context(&self) -> SkContext {
        self.size_context(true)
    }

    fn size_context(&self, download_size: bool) -> SkContext {
        let mut groups = Vec::new();
        let mut package_size: u64;
        let mut runtime_size: u64 = 0;

        // Context details for package itself
        let mut package_details = Vec::new();
        package_size = if download_size {
            self.package().download_size()
        } else {
            self.package().installed_size()
        };

        let detail = self.package().size_context_detail(download_size);
        package_details.push(detail);

        // Context details for runtimes
        let mut runtime_details = Vec::new();
        for runtime in self.runtimes().snapshot() {
            let runtime: SkDryRunPackage = runtime.downcast().unwrap();
            let detail = runtime.size_context_detail(download_size);

            let size = if download_size {
                runtime.download_size()
            } else {
                runtime.installed_size()
            };

            if runtime.name().contains(&self.package().name()) {
                package_details.push(detail);
                package_size += size;
            } else {
                runtime_details.push(detail);
                runtime_size += size;
            }
        }

        // Package context group
        let description = i18n("The storage sizes are only maximum values. Actual usage will most likely be significantly lower due to deduplication of data.");
        let group = SkContextDetailGroup::new(None, Some(&description));
        group.add_details(&package_details);
        groups.push(group);

        // Runtimes context group
        if runtime_size != 0 {
            let description = i18n("These components are shared with other applications, and only need to be downloaded once.");
            let group = SkContextDetailGroup::new(None, Some(&description));
            group.add_details(&runtime_details);
            groups.push(group);
        }

        // Context summary
        let total_size = package_size + runtime_size;
        let total_size_str = glib::format_size(total_size);
        let runtime_size_str = glib::format_size(runtime_size);
        let summary = if download_size {
            // Download size
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
                SkContextDetailKind::Icon,
                "folder-download-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        } else {
            // Installed size
            let title = if self.package().extra_data_source().is_some() {
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
                SkContextDetailKind::Icon,
                "drive-harddisk-system-symbolic",
                SkContextDetailLevel::Neutral,
                &title,
                &descr,
            )
        };

        let model = SkContextDetailGroupModel::new();
        model.add_groups(&groups);
        SkContext::new(&summary, &model)
    }

    fn data(&self) -> DryRun {
        self.imp().data.get().unwrap().clone()
    }
}
