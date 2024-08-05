// Souk - worker.rs
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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use appstream::Component;
use async_std::channel::Sender;
use flatpak::functions::system_installations;
use flatpak::prelude::*;
use flatpak::{Installation, Ref, RefKind, Remote};
use gio::Cancellable;
use glib::{clone, Downgrade};
use gtk::{gio, glib};
use indexmap::IndexMap;
use xb::prelude::*;

use crate::shared::flatpak::dry_run::DryRunPackage;
use crate::shared::flatpak::info::RemoteInfo;
use crate::shared::task::response::{OperationActivity, OperationStatus, TaskResponse, TaskResult};
use crate::shared::task::{AppstreamTask, AppstreamTaskKind};
use crate::shared::{path, WorkerError};

#[derive(Debug, Clone, Downgrade)]
pub struct AppstreamWorker {
    sender: Arc<Sender<TaskResponse>>,
}

impl AppstreamWorker {
    pub fn new(sender: Sender<TaskResponse>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn process_task(&self, task: AppstreamTask) {
        let result = match &task.kind {
            AppstreamTaskKind::Ensure => self.ensure(&task),
            AppstreamTaskKind::Update => self.update(&task),
            _ => return,
        };

        if let Err(err) = result {
            let result = TaskResult::Error(Box::new(err));
            let response = TaskResponse::new_result(task.into(), result);
            self.sender.try_send(response).unwrap();
        }
    }

    pub fn cancel_task(&self, _task_uuid: &str) {
        // Right now there is no need to make appstream tasks cancellable.
        // Can be added later if necessary.
        unimplemented!()
    }

    /// Ensures that the silo exists and that all Flatpak remotes are included
    fn ensure(&self, task: &AppstreamTask) -> Result<xb::Silo, WorkerError> {
        debug!("Ensure that silo exists with all remotes...");

        let xmlb = gio::File::for_path(path::APPSTREAM_CACHE.clone());
        let silo = xb::Silo::new();

        if silo
            .load_from_file(&xmlb, xb::SiloLoadFlags::NONE, Cancellable::NONE)
            .is_ok()
        {
            // Ensure that all known remotes are available from that silo
            let mut is_missing_remote = false;

            let installations = Self::all_installations();
            'outer: for inst in installations {
                for remote in inst.list_remotes(Cancellable::NONE)? {
                    if Self::query_remote(&silo, &remote).is_none() {
                        debug!(
                            "Silo is missing remote {:?}",
                            remote.name().unwrap_or_default()
                        );
                        is_missing_remote = true;
                        break 'outer;
                    }
                }
            }

            if !is_missing_remote {
                debug!("Found silo with all remotes. Nothing to do.");
                if task.kind != AppstreamTaskKind::Dependency {
                    let response = TaskResponse::new_result(task.clone().into(), TaskResult::Done);
                    self.sender.try_send(response).unwrap();
                }

                return Ok(silo);
            }
        } else {
            debug!("Could not load silo from file, may not exist yet.");
        }

        self.update(task)
    }

    /// Updates the appstream silo, no matter if it's up to date or not
    // TODO: Prevent potential race condition here (fresh startup, xmlb gets
    // generated, sideloading tries to access it at the same time)
    fn update(&self, task: &AppstreamTask) -> Result<xb::Silo, WorkerError> {
        debug!("Update silo...");

        let mut remotes: IndexMap<String, (Remote, Installation)> = IndexMap::new();
        let mut op_activities = Vec::new();

        for inst in Self::all_installations() {
            debug!(
                "Querying remotes for installation {:?}.",
                inst.id().unwrap()
            );
            for remote in inst.list_remotes(Cancellable::NONE)? {
                // It's possible that a same remote exists in different installations, therefore
                // we create an unique identifier (hash) for each, which gets used to
                // unambiguously identify remotes (no, we cannot use the remote
                // name, since it could point to different repository urls in
                // different installations)
                let remote_hash = Self::remote_hash(&remote);
                if remotes.contains_key(&remote_hash) {
                    debug!("Skip remote {:?}: Already added.", remote.name().unwrap());
                } else {
                    remotes.insert(remote_hash, (remote.clone(), inst.clone()));

                    let remote_info: RemoteInfo = RemoteInfo::from_flatpak(&remote, &inst);
                    let activity = OperationActivity::new_appstream(
                        Some(remote_info),
                        OperationStatus::Pending,
                    );
                    op_activities.push(activity);
                }
            }
        }

        let xmlb_activity = OperationActivity::new_appstream(None, OperationStatus::Pending);
        op_activities.push(xmlb_activity);

        let response = TaskResponse::new_activity(task.clone().into(), op_activities.clone());
        self.sender.try_send(response).unwrap();

        let builder = xb::Builder::new();
        for locale in glib::language_names() {
            builder.add_locale(&locale);
        }

        let mut imported_source = false;
        for (remote_hash, (remote, inst)) in &remotes {
            let remote_info = Some(RemoteInfo::from_flatpak(remote, inst));
            let activity = OperationActivity::new_appstream(remote_info, OperationStatus::Updating);
            let response = TaskResponse::new_activity(task.clone().into(), vec![activity]);
            self.sender.try_send(response).unwrap();

            match Self::remote_builder_source(remote, inst) {
                Ok(source) => {
                    builder.import_source(&source);
                    imported_source = true;
                }
                Err(err) => {
                    warn!(
                        "Skip remote {:?}: {}",
                        remote.name().unwrap(),
                        err.to_string()
                    );

                    // Create empty placeholder entry
                    let node = xb::BuilderNode::new("components");
                    node.set_attr("origin", remote_hash);
                    node.set_attr("error", &err.to_string());
                    builder.import_node(&node);
                }
            }

            let remote_info = Some(RemoteInfo::from_flatpak(remote, inst));
            let activity = OperationActivity::new_appstream(remote_info, OperationStatus::Done);
            let response = TaskResponse::new_activity(task.clone().into(), vec![activity]);
            self.sender.try_send(response).unwrap();
        }

        if !imported_source {
            let glib_error = glib::Error::new(flatpak::Error::Aborted, "");
            warn!("Unable to retrieve Flatpak appstream data.");
            return Err(glib_error.into());
        }

        debug!("Ensure compiled xmlb is up to date.");
        let xmlb_activity = OperationActivity::new_appstream(None, OperationStatus::Processing);
        let response = TaskResponse::new_activity(task.clone().into(), vec![xmlb_activity]);
        self.sender.try_send(response).unwrap();

        let xmlb = gio::File::for_path(path::APPSTREAM_CACHE.clone());
        let silo = builder.ensure(
            &xmlb,
            xb::BuilderCompileFlags::IGNORE_INVALID.union(xb::BuilderCompileFlags::SINGLE_LANG),
            Cancellable::NONE,
        )?;

        debug!("Done.");
        let xmlb_activity = OperationActivity::new_appstream(None, OperationStatus::Done);
        let response = TaskResponse::new_activity(task.clone().into(), vec![xmlb_activity]);
        self.sender.try_send(response).unwrap();

        if task.kind != AppstreamTaskKind::Dependency {
            let response = TaskResponse::new_result(task.clone().into(), TaskResult::Done);
            self.sender.try_send(response).unwrap();
        }

        Ok(silo)
    }

    fn remote_builder_source(
        remote: &Remote,
        installation: &Installation,
    ) -> Result<xb::BuilderSource, WorkerError> {
        let source = xb::BuilderSource::new();
        let remote_name = remote.name().unwrap();

        let appstream_dir = remote
            .appstream_dir(None)
            .ok_or::<WorkerError>(glib::Error::new(flatpak::Error::Skipped, "").into())?;
        let appstream_file = appstream_dir.child("appstream.xml.gz");

        debug!(
            "Sync appstream data for remote {:?} ({:?})",
            remote_name,
            appstream_file.path().unwrap()
        );
        // TODO: Make use of the `out_changed` field here, which isn't exposed in
        // libflatpak-rs yet
        installation.update_appstream_full_sync(&remote_name, None, None, Cancellable::NONE)?;

        source.load_file(
            &appstream_file,
            xb::BuilderSourceFlags::LITERAL_TEXT,
            Cancellable::NONE,
        )?;

        Self::add_source_fixups(&source, remote);

        Ok(source)
    }

    pub(super) fn set_dry_run_package_appstream(
        &self,
        task: &AppstreamTask,
        package: &mut DryRunPackage,
        ref_str: &str,
        remote: &Remote,
        installation: &Installation,
    ) -> Result<(), WorkerError> {
        let remote_name = remote.name().unwrap_or_default();
        debug!(
            "Retrieve appstream data for dry run package: {} (\"{remote_name}\")",
            package.info.ref_
        );
        let default_silo = self.ensure(task)?;

        let silo = if Self::query_remote(&default_silo, remote).is_some() {
            debug!("Remote \"{remote_name}\" is known, load cached silo from file.");
            default_silo
        } else {
            debug!("Remote \"{remote_name}\" is not known yet, create new temporary silo.");

            let installation_path = installation.path().unwrap();
            let xmlb = installation_path.child("appstream.xmlb");
            let silo = xb::Silo::new();

            // Check if a xmlb silo for the dry run installation already exists
            let res = silo.load_from_file(&xmlb, xb::SiloLoadFlags::NONE, Cancellable::NONE);
            if res.is_ok() {
                debug!("Use already created silo for dry-run installation.");
                silo
            } else {
                debug!("Compile new temporary silo for dry-run installation.");
                let builder = xb::Builder::new();

                let source = Self::remote_builder_source(remote, installation)?;
                builder.import_source(&source);

                let new_silo = builder.compile(xb::BuilderCompileFlags::NONE, Cancellable::NONE)?;
                new_silo.save_to_file(&xmlb, Cancellable::NONE)?;

                new_silo
            }
        };

        let ref_ = Ref::parse(ref_str).unwrap();
        let mut kind = if ref_.kind() == RefKind::App {
            "app"
        } else {
            "runtime"
        };
        let mut name = ref_.name().unwrap().to_string();
        let arch = ref_.arch().unwrap().to_string();
        let branch = ref_.branch().unwrap().to_string();

        // Those Flatpak subrefs usually don't include appstream data.
        // So we strip the suffixes, and retrieve the appstream data of the actual ref.
        // We use here the same subrefs as Flatpak, see:
        // https://github.com/flatpak/flatpak/blob/600e18567c538ecd306d021534dbb418dc490676/common/flatpak-ref-utils.c#L451
        // TODO: Do we really want to display subrefs in UI?
        if name.ends_with(".Locale") || name.ends_with(".Debug") || name.ends_with(".Sources") {
            name = name.trim_end_matches(".Locale").into();
            name = name.trim_end_matches(".Debug").into();
            name = name.trim_end_matches(".Sources").into();
            kind = "app";
        }

        // Component
        let query = format!("{}/{}/{}/{}", kind, name, arch, branch);
        if let Some(component) = Self::query_component(&silo, &query, remote)? {
            package.appstream_component = Some(serde_json::to_string(&component).unwrap());
        }

        // Icon
        if let Some(node) = Self::query_remote(&silo, remote) {
            let appstream_path = gio::File::for_parse_name(&node.attr("path"));
            let icon_file = appstream_path.child(format!("icons/128x128/{}.png", name));

            if let Ok((bytes, _)) = icon_file.load_bytes(Cancellable::NONE) {
                package.icon = Some(bytes.to_vec());
            }
        } else {
            warn!("Unable to set icon for dry-run package, remote does not exist in silo.");
        }

        Ok(())
    }

    fn query_component(
        silo: &xb::Silo,
        ref_str: &str,
        remote: &Remote,
    ) -> Result<Option<Component>, WorkerError> {
        let ref_escaped = ref_str.replace('/', r"\/");
        let remote_hash = Self::remote_hash(remote);
        let xpath = format!(
            "components[@origin='{remote_hash}']/component/bundle[text()='{ref_escaped}']/.."
        );

        if let Ok(node) = silo.query_first(&xpath) {
            let xml = node.export(xb::NodeExportFlags::NONE).unwrap().to_string();
            let element = xmltree::Element::parse(xml.as_bytes()).unwrap();

            if let Ok(component) = Component::try_from(&element) {
                return Ok(Some(component));
            } else {
                warn!("Couldn't find appstream component for {ref_str}");
            }
        }

        Ok(None)
    }

    fn query_remote(silo: &xb::Silo, remote: &Remote) -> Option<xb::Node> {
        let remote_hash = Self::remote_hash(remote);
        let xpath = format!("components[@origin='{remote_hash}']");
        silo.query_first(&xpath).ok()
    }

    // Based on the gnome-software fixups
    // https://gitlab.gnome.org/GNOME/gnome-software/-/blob/35e2d0e4191d0c81bf48e5f05bbb1c110572f917/plugins/flatpak/gs-flatpak.c#L677
    fn add_source_fixups(source: &xb::BuilderSource, remote: &Remote) {
        // Ensure the <id> matches the flatpak ref ID
        let fixup = xb::BuilderFixup::new("FixIdDesktopSuffix", |_, node, _| {
            if node.element().is_some() {
                let id = node.child("id", None);
                let bundle = node.child("bundle", None);

                if id.is_none() || bundle.is_none() {
                    return true;
                }
                let id = id.unwrap();
                let bundle = bundle.unwrap();

                let bundle_txt = bundle.text();
                let split: Vec<&str> = bundle_txt.split('/').collect();
                if split.len() != 4 {
                    return true;
                }

                let component_id = id.text();
                let correct_id = split[1];

                if component_id != correct_id {
                    id.set_text(Some(correct_id));

                    // Add the "wrong" id to "provides", so we can find the component by the wrong
                    // id as well
                    let provides = match node.child("provides", None) {
                        Some(provides) => provides,
                        None => {
                            let n = xb::BuilderNode::new("provides");
                            node.add_child(&n);
                            n
                        }
                    };

                    if provides.child("id", Some(&component_id)).is_none() {
                        let id_node = xb::BuilderNode::new("id");
                        id_node.set_text(Some(&component_id));
                        provides.add_child(&id_node);
                    };
                }
            }

            true
        });
        fixup.set_max_depth(2);
        source.add_fixup(&fixup);

        // Add tokens to allow much faster searching
        let fixup = xb::BuilderFixup::new("TextTokenize", |_, node, _| {
            if let Some(element) = node.element() {
                let tokens = ["id", "keyword", "launchable", "mimetype", "name", "summary"];

                if tokens.contains(&element.as_str()) {
                    node.tokenize_text();
                }
            }

            true
        });
        fixup.set_max_depth(2);
        source.add_fixup(&fixup);

        // Set origin / apstream path
        let fixup = xb::BuilderFixup::new(
            "SetOrigin",
            clone!(
                #[strong]
                remote,
                move |_, node, _| {
                    if let Some(element) = node.element() {
                        if element.as_str() == "components" {
                            let hash = Self::remote_hash(&remote);
                            let path: String = remote
                                .appstream_dir(None)
                                .map(|f| f.parse_name().to_string())
                                .unwrap_or_default();

                            node.set_attr("origin", &hash);
                            node.set_attr("path", &path);
                        }
                    }

                    true
                }
            ),
        );
        fixup.set_max_depth(1);
        source.add_fixup(&fixup);

        // TODO: Add `FilterNoEnumerate`?
    }

    fn remote_hash(remote: &Remote) -> String {
        // In theory we could use the remote collection id here - which unfortunately
        // isn't deployed widely for remotes yet :(
        let mut hasher = DefaultHasher::new();

        if let Some(url) = remote.url() {
            url.to_string().hash(&mut hasher);
        } else {
            // fallback to combination out of name and title (eg. "devel-origin" and
            // "souk_x86_64.flatpak")
            remote.name().unwrap_or_default().hash(&mut hasher);
            remote.title().unwrap_or_default().hash(&mut hasher);
        }

        hasher.finish().to_string()
    }

    fn all_installations() -> Vec<Installation> {
        let mut installations = Vec::new();

        // User installation
        let mut user_path = glib::home_dir();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        let file = gio::File::for_path(user_path);
        let inst = Installation::for_path(&file, true, Cancellable::NONE).unwrap();
        installations.push(inst);

        // System + extra installations
        let mut extra_inst = system_installations(Cancellable::NONE).unwrap();
        installations.append(&mut extra_inst);

        installations
    }
}
