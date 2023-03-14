// Souk - remote.rs
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

use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use glib::{ParamSpec, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::unsync::OnceCell;

use super::SkInstallation;
use crate::main::SkApplication;
use crate::shared::flatpak::info::RemoteInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkRemote)]
    pub struct SkRemote {
        #[property(get)]
        title: RefCell<String>,
        #[property(get)]
        description: RefCell<String>,
        #[property(get)]
        comment: RefCell<String>,
        #[property(get)]
        homepage: RefCell<String>,
        #[property(get)]
        icon: RefCell<String>,
        #[property(get)]
        installation: OnceCell<Option<SkInstallation>>,
        #[property(name = "name", get, type = String, member = name)]
        #[property(name = "repository-url", get, type = String, member = repository_url)]
        #[property(get, set, construct_only)]
        info: OnceCell<RemoteInfo>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkRemote {
        const NAME: &'static str = "SkRemote";
        type Type = super::SkRemote;
    }

    impl ObjectImpl for SkRemote {
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

            let info = self.obj().info();

            // Try to convert the `RemoteInfo` into a Flatpak `Remote` object.
            // This only works, when the `RemoteInfo` has `repo_bytes` set.
            let flatpak_remote: Option<Remote> = info.clone().try_into().ok();
            if let Some(flatpak_remote) = flatpak_remote {
                self.set_remote_data(&flatpak_remote);
            }

            if let Some(inst_info) = &info.installation {
                let flatpak_inst = Installation::from(inst_info);
                if let Ok(flatpak_remote) =
                    flatpak_inst.remote_by_name(&info.name, gio::Cancellable::NONE)
                {
                    self.set_remote_data(&flatpak_remote);
                }

                let installations = SkApplication::default().worker().installations();
                let installation = installations
                    .installation(inst_info)
                    .expect("Unknown Flatpak installation");
                self.installation.set(Some(installation)).unwrap();
            } else {
                self.installation.set(None).unwrap();
            }
        }
    }

    impl SkRemote {
        fn set_remote_data(&self, remote: &Remote) {
            *self.title.borrow_mut() = remote.title().unwrap_or_default().to_string();
            *self.description.borrow_mut() = remote.description().unwrap_or_default().to_string();
            *self.comment.borrow_mut() = remote.comment().unwrap_or_default().to_string();
            *self.homepage.borrow_mut() = remote.homepage().unwrap_or_default().to_string();
            *self.icon.borrow_mut() = remote.icon().unwrap_or_default().to_string();
        }
    }
}

glib::wrapper! {
    pub struct SkRemote(ObjectSubclass<imp::SkRemote>);
}

impl SkRemote {
    pub fn new(info: &RemoteInfo) -> Self {
        glib::Object::builder().property("info", info).build()
    }
}
