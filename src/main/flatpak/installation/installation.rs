// Souk - installation.rs
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

use async_std::process::Command;
use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use gio::{Cancellable, File, FileMonitor};
use glib::{clone, ParamSpec, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::installation::{SkRemote, SkRemoteModel};
use crate::main::flatpak::package::{SkPackage, SkPackageModel};
use crate::main::i18n::i18n;
use crate::shared::flatpak::info::{InstallationInfo, PackageInfo, RemoteInfo};

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkInstallation)]
    pub struct SkInstallation {
        #[property(get)]
        name: OnceCell<String>,
        #[property(get)]
        title: OnceCell<String>,
        #[property(get)]
        description: OnceCell<String>,
        #[property(get)]
        icon_name: OnceCell<String>,
        #[property(get)]
        remotes: SkRemoteModel,
        #[property(get)]
        packages: SkPackageModel,
        #[property(name = "path", get = Self::path, type = File)]
        #[property(name = "is-user", get, type = bool, member = is_user)]
        #[property(get, set, construct_only)]
        info: OnceCell<InstallationInfo>,

        monitor: OnceCell<FileMonitor>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallation {
        const NAME: &'static str = "SkInstallation";
        type Type = super::SkInstallation;
    }

    impl ObjectImpl for SkInstallation {
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
            let obj = self.obj();

            self.name.set(info.name.clone()).unwrap();

            // Create file monitor to detect changes in the installation (eg. ref
            // in/uninstall, remote changes)
            let f_inst = Installation::from(&info);
            let monitor = f_inst.create_monitor(Cancellable::NONE).unwrap();
            monitor.connect_changed(clone!(@weak obj as this => move |_,_,_,_|{
                debug!("Detected change in Flatpak installation.");
                if let Err(err) = this.refresh(){
                    error!("Unable to refresh Flatpak installation: {}", err.to_string());
                    // TODO: Should this be exposed in the UI?
                }
            }));
            self.monitor.set(monitor).unwrap();

            // Set a more user friendly installation title, a description and an icon which
            // can be used in UIs.
            if info.name == "default" && !info.is_user {
                // Default system installation
                let title = i18n("System");
                self.title.set(title).unwrap();

                let description = i18n("All users on this computer");
                self.description.set(description).unwrap();

                self.icon_name.set("computer-symbolic".into()).unwrap();
            } else if info.name == "user" && info.is_user {
                // Default user installation
                let title = i18n("User");
                self.title.set(title).unwrap();

                let description = i18n("Only currently logged in user");
                self.description.set(description).unwrap();

                self.icon_name.set("person-symbolic".into()).unwrap();
            } else {
                let title = if let Some(string) = Installation::from(&info).display_name() {
                    string.to_string()
                } else {
                    i18n("Flatpak Installation")
                };

                self.title.set(title).unwrap();
                if info.is_user {
                    self.description.set(i18n("User Installation")).unwrap();
                } else {
                    self.description.set(i18n("System Installation")).unwrap();
                }

                self.icon_name
                    .set("drive-harddisk-symbolic".into())
                    .unwrap();
            }
        }
    }

    impl SkInstallation {
        fn path(&self) -> File {
            File::for_parse_name(&self.obj().info().path)
        }
    }
}

glib::wrapper! {
    pub struct SkInstallation(ObjectSubclass<imp::SkInstallation>);
}

impl SkInstallation {
    pub(super) fn new(info: &InstallationInfo) -> Self {
        glib::Object::builder().property("info", info).build()
    }

    pub fn launch_app(&self, app: &SkPackage) {
        debug!(
            "Launch app from installation \"{}\": {}",
            self.name(),
            app.name()
        );

        let installation = if self.name() == "user" {
            "--user".into()
        } else {
            format!("--installation={}", self.name())
        };

        if let Err(err) = Command::new("flatpak-spawn")
            .arg("--host")
            .arg("flatpak")
            .arg("run")
            .arg(installation)
            .arg(app.name())
            .spawn()
        {
            error!("Unable to launch app: {}", err.to_string());
        }
    }

    pub fn add_remote(&self, remote: &SkRemote) -> Result<(), Error> {
        debug!(
            "Adding remote \"{}\" to installation \"{}\"",
            remote.name(),
            self.name()
        );

        let f_remote: Remote = remote.to_owned().info().try_into()?;

        let f_inst = Installation::from(&self.info());
        f_inst.add_remote(&f_remote, false, Cancellable::NONE)?;

        self.refresh()?;

        Ok(())
    }

    pub fn refresh(&self) -> Result<(), Error> {
        debug!(
            "Refresh Flatpak \"{}\" ({}) installation...",
            self.title(),
            self.name()
        );

        let f_inst = Installation::from(&self.info());

        let f_remotes = f_inst.list_remotes(Cancellable::NONE)?;
        let mut remotes = Vec::new();
        for f_remote in &f_remotes {
            let remote_info = RemoteInfo::from_flatpak(f_remote, &f_inst);
            remotes.push(remote_info);
        }
        self.remotes().set_remotes(remotes);

        let f_packages = f_inst.list_installed_refs(Cancellable::NONE)?;
        let mut packages = Vec::new();
        for f_package in &f_packages {
            let f_remote = f_inst
                .remote_by_name(&f_package.origin().unwrap(), Cancellable::NONE)
                .unwrap();
            let package_info = PackageInfo::from_flatpak(f_package, &f_remote, &f_inst);
            packages.push(package_info);
        }
        self.packages().set_packages(packages);

        Ok(())
    }
}
