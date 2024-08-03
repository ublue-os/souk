// Souk - installation_model.rs
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

use std::cell::RefCell;
use std::convert::TryInto;

use flatpak::functions::system_installations;
use flatpak::Installation;
use gtk::gio::Cancellable;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::main::flatpak::installation::SkInstallation;
use crate::shared::flatpak::info::InstallationInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkInstallationModel {
        pub map: RefCell<IndexMap<InstallationInfo, SkInstallation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallationModel {
        const NAME: &'static str = "SkInstallationModel";
        type Type = super::SkInstallationModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SkInstallationModel {
        fn constructed(&self) {
            self.parent_constructed();

            // System Installation
            let f_inst = Installation::new_system(Cancellable::NONE).unwrap();
            let info = InstallationInfo::from(&f_inst);
            self.add_info(&info);

            // User Installation
            let mut user_path = glib::home_dir();
            user_path.push(".local");
            user_path.push("share");
            user_path.push("flatpak");
            let file = gio::File::for_path(user_path);

            let f_inst = Installation::for_path(&file, true, Cancellable::NONE).unwrap();
            let info = InstallationInfo::from(&f_inst);
            self.add_info(&info);
        }
    }

    impl ListModelImpl for SkInstallationModel {
        fn item_type(&self) -> glib::Type {
            SkInstallation::static_type()
        }

        fn n_items(&self) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }

    impl SkInstallationModel {
        pub fn add_info(&self, info: &InstallationInfo) {
            let pos = {
                let mut map = self.map.borrow_mut();
                if map.contains_key(info) {
                    return;
                }

                let sk_inst = SkInstallation::new(info);
                map.insert(info.clone(), sk_inst);
                (map.len() - 1) as u32
            };

            self.obj().items_changed(pos, 0, 1);
        }

        pub fn remove_info(&self, info: &InstallationInfo) {
            let pos = {
                let mut map = self.map.borrow_mut();
                match map.get_index_of(info) {
                    Some(pos) => {
                        map.swap_remove(info);
                        Some(pos)
                    }
                    None => {
                        warn!(
                            "Unable to remove installation {:?}, not found in model",
                            info.name
                        );
                        None
                    }
                }
            };

            if let Some(pos) = pos {
                self.obj().items_changed(pos.try_into().unwrap(), 1, 0);
            }
        }
    }
}

glib::wrapper! {
    pub struct SkInstallationModel(ObjectSubclass<imp::SkInstallationModel>) @implements gio::ListModel;
}

impl SkInstallationModel {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn refresh(&self) {
        debug!("Refresh Flatpak installations...");
        let imp = self.imp();

        let extra_flatpak_installations = system_installations(Cancellable::NONE).unwrap();
        let mut extra_infos = Vec::new();
        for extra_flatpak_installation in extra_flatpak_installations {
            let info = InstallationInfo::from(&extra_flatpak_installation);
            imp.add_info(&info);
            extra_infos.push(info);
        }

        let map = imp.map.borrow().clone();
        for (info, sk_inst) in map.iter() {
            if info.name != "user" && !extra_infos.contains(info) {
                imp.remove_info(info);
            } else {
                sk_inst.refresh();
            }
        }
    }

    pub fn installation(&self, info: &InstallationInfo) -> Option<SkInstallation> {
        self.imp().map.borrow().get(info).cloned()
    }

    /// Returns the [`SkInstallation`] with the most installed refs.
    pub fn preferred(&self) -> SkInstallation {
        let imp = self.imp();
        let map = imp.map.borrow();

        let mut top_count = 0;
        let mut preferred = None;

        for sk_inst in map.values() {
            let count = sk_inst.packages().n_items();

            if count >= top_count {
                top_count = count;
                preferred = Some(sk_inst);
            }
        }

        preferred.unwrap().clone()
    }
}

impl Default for SkInstallationModel {
    fn default() -> Self {
        Self::new()
    }
}
