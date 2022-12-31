// Souk - installation.rs
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

use async_std::process::Command;
use flatpak::prelude::*;
use flatpak::{Installation, Remote};
use gio::{Cancellable, File, FileMonitor};
use glib::{
    clone, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, ToValue,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::main::error::Error;
use crate::main::flatpak::installation::{SkRemote, SkRemoteModel};
use crate::main::flatpak::package::{SkPackage, SkPackageModel};
use crate::main::i18n::i18n;
use crate::shared::info::{InstallationInfo, PackageInfo, RemoteInfo};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SkInstallation {
        pub info: OnceCell<InstallationInfo>,
        pub monitor: OnceCell<FileMonitor>,

        pub name: OnceCell<String>,
        pub title: OnceCell<String>,
        pub description: OnceCell<String>,
        pub icon_name: OnceCell<String>,
        pub is_user: OnceCell<bool>,
        pub path: OnceCell<File>,

        pub remotes: SkRemoteModel,
        pub packages: SkPackageModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkInstallation {
        const NAME: &'static str = "SkInstallation";
        type Type = super::SkInstallation;
    }

    impl ObjectImpl for SkInstallation {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("title", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("description", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("icon-name", "", "", None, ParamFlags::READABLE),
                    ParamSpecBoolean::new("is-user", "", "", false, ParamFlags::READABLE),
                    ParamSpecObject::new("path", "", "", File::static_type(), ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "remotes",
                        "",
                        "",
                        SkRemoteModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "packages",
                        "",
                        "",
                        SkPackageModel::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "title" => obj.title().to_value(),
                "description" => obj.description().to_value(),
                "icon-name" => obj.icon_name().to_value(),
                "is-user" => obj.is_user().to_value(),
                "path" => obj.path().to_value(),
                "remotes" => obj.remotes().to_value(),
                "packages" => obj.packages().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SkInstallation(ObjectSubclass<imp::SkInstallation>);
}

impl SkInstallation {
    pub(super) fn new(info: &InstallationInfo) -> Self {
        let installation: Self = glib::Object::new(&[]).unwrap();
        let imp = installation.imp();

        imp.info.set(info.clone()).unwrap();

        // Create file monitor to detect changes in the installation (eg. ref
        // in/uninstall, remote changes)
        let f_inst = Installation::from(info);
        let monitor = f_inst.create_monitor(Cancellable::NONE).unwrap();
        monitor.connect_changed(clone!(@weak installation as this => move |_,_,_,_|{
            debug!("Detected change in Flatpak installation.");
            if let Err(err) = this.refresh(){
                error!("Unable to refresh Flatpak installation: {}", err.to_string());
                // TODO: Should this be exposed in the UI?
            }
        }));
        imp.monitor.set(monitor).unwrap();

        imp.name.set(info.name.clone()).unwrap();
        imp.is_user.set(info.is_user).unwrap();
        let path = File::for_parse_name(&info.path);
        imp.path.set(path).unwrap();

        // Set a more user friendly installation title, a description and an icon which
        // can be used in UIs.
        if info.name == "default" && !info.is_user {
            // Default system installation
            let title = i18n("System");
            imp.title.set(title).unwrap();

            let description = i18n("All users on this computer");
            imp.description.set(description).unwrap();

            imp.icon_name.set("computer-symbolic".into()).unwrap();
        } else if info.name == "user" && info.is_user {
            // Default user installation
            let title = i18n("User");
            imp.title.set(title).unwrap();

            let description = i18n("Only currently logged in user");
            imp.description.set(description).unwrap();

            imp.icon_name.set("person-symbolic".into()).unwrap();
        } else {
            let title = if let Some(string) = Installation::from(info).display_name() {
                string.to_string()
            } else {
                i18n("Flatpak Installation")
            };

            imp.title.set(title).unwrap();
            if info.is_user {
                imp.description.set(i18n("User Installation")).unwrap();
            } else {
                imp.description.set(i18n("System Installation")).unwrap();
            }

            imp.icon_name.set("drive-harddisk-symbolic".into()).unwrap();
        }

        installation
    }

    pub fn name(&self) -> String {
        self.imp().name.get().unwrap().to_string()
    }

    pub fn title(&self) -> String {
        self.imp().title.get().unwrap().to_string()
    }

    pub fn description(&self) -> String {
        self.imp().description.get().unwrap().to_string()
    }

    pub fn icon_name(&self) -> String {
        self.imp().icon_name.get().unwrap().to_string()
    }

    pub fn is_user(&self) -> bool {
        *self.imp().is_user.get().unwrap()
    }

    pub fn path(&self) -> File {
        self.imp().path.get().unwrap().clone()
    }

    pub fn remotes(&self) -> SkRemoteModel {
        self.imp().remotes.clone()
    }

    pub fn packages(&self) -> SkPackageModel {
        self.imp().packages.clone()
    }

    pub fn info(&self) -> InstallationInfo {
        self.imp().info.get().unwrap().clone()
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
            let remote_info = RemoteInfo::from_flatpak(f_remote, Some(&f_inst));
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
