// Souk - operation.rs
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

use std::cell::{Cell, OnceCell, RefCell};

use glib::{ParamSpec, Properties};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::main::flatpak::installation::SkRemote;
use crate::main::flatpak::package::SkPackage;
use crate::main::task::{SkOperationKind, SkOperationStatus};
use crate::shared::appstream::AppstreamOperationKind;
use crate::shared::flatpak::FlatpakOperationKind;
use crate::shared::task::response::OperationActivity;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkOperation)]
    pub struct SkOperation {
        // Static values
        #[property(get, set, construct_only)]
        index: OnceCell<u32>,
        #[property(get, set, construct_only)]
        identifier: OnceCell<String>,
        #[property(get, set, construct_only, builder(SkOperationKind::None))]
        kind: OnceCell<SkOperationKind>,
        #[property(get, set, construct_only)]
        package: RefCell<Option<SkPackage>>,
        #[property(get, set, construct_only)]
        remote: RefCell<Option<SkRemote>>,

        // Dynamic values
        #[property(get, builder(SkOperationStatus::Pending))]
        pub status: RefCell<SkOperationStatus>,
        #[property(get)]
        pub progress: Cell<f32>,
        #[property(get)]
        pub download_rate: Cell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkOperation {
        const NAME: &'static str = "SkOperation";
        type Type = super::SkOperation;
    }

    impl ObjectImpl for SkOperation {
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
}

glib::wrapper! {
    pub struct SkOperation(ObjectSubclass<imp::SkOperation>);
}

impl SkOperation {
    pub fn new(index: u32, activity: &OperationActivity) -> Self {
        let kind = if activity.flatpak_operation != FlatpakOperationKind::None {
            SkOperationKind::from(activity.flatpak_operation.clone())
        } else if activity.appstream_operation != AppstreamOperationKind::None {
            SkOperationKind::from(activity.appstream_operation.clone())
        } else {
            warn!("Unable to determine operation kind");
            SkOperationKind::None
        };

        let package = activity.package.as_ref().map(SkPackage::new);
        let remote = activity.remote.as_ref().map(SkRemote::new);

        let operation: Self = glib::Object::builder()
            .property("index", index)
            .property("identifier", activity.identifier())
            .property("kind", kind)
            .property("package", package)
            .property("remote", remote)
            .build();

        operation.handle_activity(activity);
        operation
    }

    pub fn handle_activity(&self, activity: &OperationActivity) {
        let imp = self.imp();

        // status
        let status = SkOperationStatus::from(activity.status.clone());
        if self.status() != status {
            *imp.status.borrow_mut() = status;
            self.notify("status");
        }

        // progress
        let progress = activity.progress as f32 / 100.0;
        if self.progress() != progress {
            imp.progress.set(progress);
            self.notify("progress");
        }

        // download rate
        if self.download_rate() != activity.download_rate {
            imp.download_rate.set(activity.download_rate);
            self.notify("download-rate");
        }
    }
}
