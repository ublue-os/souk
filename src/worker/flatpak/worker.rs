// Souk - worker.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ::appstream::Collection;
use async_std::channel::Sender;
use flatpak::prelude::*;
use flatpak::{BundleRef, Installation, Ref, Remote, Transaction, TransactionOperationType};
use gio::prelude::*;
use gio::Cancellable;
use glib::{clone, Downgrade, KeyFile};
use gtk::{gio, glib};
use isahc::ReadResponseExt;

use super::{DryRunResult, DryRunRuntime};
use crate::shared::info::{InstallationInfo, RemoteInfo};
use crate::shared::task::{FlatpakOperationType, FlatpakTask, Response, TaskResult, TaskStep};
use crate::worker::{appstream, WorkerError};

#[derive(Debug, Clone, Downgrade)]
pub struct FlatpakWorker {
    transactions: Arc<Mutex<HashMap<String, Cancellable>>>,
    sender: Arc<Sender<Response>>,
}

impl FlatpakWorker {
    pub fn new(sender: Sender<Response>) -> Self {
        Self {
            transactions: Arc::default(),
            sender: Arc::new(sender),
        }
    }

    pub fn process_task(&self, task: FlatpakTask, task_uuid: &str) {
        let result = match task.operation_type {
            FlatpakOperationType::Install => {
                if task.dry_run {
                    unimplemented!();
                } else {
                    self.install_flatpak(
                        task_uuid,
                        task.ref_.as_ref().unwrap(),
                        task.remote.as_ref().unwrap(),
                        &task.installation,
                        task.uninstall_before_install,
                    )
                }
            }
            FlatpakOperationType::InstallBundleFile => {
                if task.dry_run {
                    self.install_flatpak_bundle_file_dry_run(
                        task_uuid,
                        task.path.as_ref().unwrap(),
                        &task.installation,
                    )
                } else {
                    self.install_flatpak_bundle_file(
                        task_uuid,
                        task.path.as_ref().unwrap(),
                        &task.installation,
                        task.uninstall_before_install,
                    )
                }
            }
            FlatpakOperationType::InstallRefFile => {
                if task.dry_run {
                    self.install_flatpak_ref_file_dry_run(
                        task_uuid,
                        task.path.as_ref().unwrap(),
                        &task.installation,
                    )
                } else {
                    self.install_flatpak_ref_file(
                        task_uuid,
                        task.path.as_ref().unwrap(),
                        &task.installation,
                        task.uninstall_before_install,
                    )
                }
            }
            FlatpakOperationType::Uninstall => {
                unimplemented!();
            }
        };

        if let Err(err) = result {
            // Transaction got cancelled (probably by user)
            if err == WorkerError::GLibCancelled {
                let result = TaskResult::new_cancelled();
                let response = Response::new_result(task_uuid.into(), result);
                self.sender.try_send(response).unwrap();
            } else {
                let result = TaskResult::new_error(err.message());
                let response = Response::new_result(task_uuid.into(), result);
                self.sender.try_send(response).unwrap();
            }
        }
    }

    pub fn cancel_task(&self, task_uuid: &str) {
        let transactions = self.transactions.lock().unwrap();
        if let Some(cancellable) = transactions.get(task_uuid) {
            cancellable.cancel();
        } else {
            warn!("Unable to cancel flatpak task: {}", task_uuid);
        }
    }

    fn install_flatpak(
        &self,
        task_uuid: &str,
        ref_: &str,
        remote: &RemoteInfo,
        installation_info: &InstallationInfo,
        uninstall_before_install: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak: {}", ref_);

        let transaction = self.new_transaction(installation_info, false)?;

        if uninstall_before_install {
            self.uninstall_ref(ref_, installation_info)?;
        }
        transaction.add_install(&remote.name, ref_, &[])?;

        self.run_transaction(task_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle_file(
        &self,
        task_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
        uninstall_before_install: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation_info, false)?;

        if uninstall_before_install {
            let bundle = BundleRef::new(&file)?;
            let ref_ = bundle.format_ref().unwrap();
            self.uninstall_ref(&ref_, installation_info)?;
        }
        transaction.add_install_bundle(&file, None)?;

        self.run_transaction(task_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle_file_dry_run(
        &self,
        task_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak bundle (dry run): {}", path);
        let file = gio::File::for_parse_name(path);
        let bundle = BundleRef::new(&file)?;

        let transaction = self.new_transaction(installation_info, true)?;
        transaction.add_install_bundle(&file, None)?;

        let mut results = self.run_dry_run_transaction(transaction, installation_info, false)?;

        // Installed bundle size
        results.installed_size = bundle.installed_size();

        // Remotes
        if let Some(runtime_repo_url) = bundle.runtime_repo_url() {
            // Download Flatpak repofile for additional remote metadata
            let bundle_remote = self.retrieve_flatpak_remote(&runtime_repo_url)?;

            let mut remotes_info = Vec::new();
            for (name, url) in &results.remotes {
                if bundle_remote.url().unwrap().as_str() == url {
                    remotes_info.push(RemoteInfo::from(&bundle_remote));
                } else {
                    remotes_info.push(RemoteInfo::new(name, url));
                }
            }
            results.remotes_info = remotes_info;
        }

        // Icon
        if let Some(bytes) = bundle.icon(128) {
            results.icon = bytes.to_vec();
        }

        // Appstream metadata
        if let Some(compressed) = bundle.appstream() {
            let collection = Collection::from_gzipped_bytes(&compressed).unwrap();
            let component = &collection.components[0];

            let json = serde_json::to_string(component).unwrap();
            results.appstream_component = Some(json).into();
        }

        let task_result = TaskResult::new_dry_run(results);
        let response = Response::new_result(task_uuid.to_string(), task_result);
        self.sender.try_send(response).unwrap();

        Ok(())
    }

    fn install_flatpak_ref_file(
        &self,
        task_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
        uninstall_before_install: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak ref: {}", path);
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;

        let transaction = self.new_transaction(installation_info, false)?;

        if uninstall_before_install {
            let keyfile = KeyFile::new();
            keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;
            let name = keyfile.value("Flatpak Ref", "Name")?;
            let branch = keyfile.value("Flatpak Ref", "Branch")?;
            let arch = flatpak::functions::default_arch().unwrap();

            let ref_ = format!("app/{name}/{arch}/{branch}");
            self.uninstall_ref(&ref_, installation_info)?;
        }
        transaction.add_install_flatpakref(&bytes)?;

        self.run_transaction(task_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_ref_file_dry_run(
        &self,
        task_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak ref (dry run): {}", path);
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;

        let transaction = self.new_transaction(installation_info, true)?;
        transaction.add_install_flatpakref(&bytes)?;

        let mut results = self.run_dry_run_transaction(transaction, installation_info, true)?;

        // Up to two remotes can be added during a *.flatpakref installation:
        // 1) `Url` value (= the repository where the ref is located)
        // 2) `RuntimeRepo` value (doesn't need to point to the same repository as
        // `Url`)
        let mut remotes_info = Vec::new();

        let keyfile = KeyFile::new();
        keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;
        let mut ref_repo_url = String::new();

        if let Ok(repo_url) = keyfile.value("Flatpak Ref", "RuntimeRepo") {
            if !repo_url.is_empty() {
                let ref_repo = self.retrieve_flatpak_remote(&repo_url)?;
                ref_repo_url = ref_repo.url().unwrap().to_string();

                if results.remotes.iter().any(|(_, url)| url == &ref_repo_url) {
                    remotes_info.push(RemoteInfo::from(&ref_repo));
                }
            }
        }

        for (name, url) in &results.remotes {
            if url != &ref_repo_url && !ref_repo_url.is_empty() {
                remotes_info.push(RemoteInfo::new(name, url));
            }
        }
        results.remotes_info = remotes_info;

        let task_result = TaskResult::new_dry_run(results);
        let response = Response::new_result(task_uuid.to_string(), task_result);
        self.sender.try_send(response).unwrap();

        Ok(())
    }

    fn run_transaction(
        &self,
        task_uuid: String,
        transaction: Transaction,
    ) -> Result<(), WorkerError> {
        transaction.connect_add_new_remote(move |_, _, _, _, _| true);

        transaction.connect_ready(
            clone!(@strong task_uuid, @weak self.sender as sender => @default-return true, move |transaction|{
                let mut steps = Vec::new();
                for operation in transaction.operations(){
                    let step = TaskStep::new_flatpak(transaction, &operation, None, false);
                    steps.push(step);
                }

                let response = Response::new_initial(task_uuid.clone(), steps);
                sender.try_send(response).unwrap();

                // Real transaction -> start (unlike dryrun)
                true
            }),
        );

        transaction.connect_new_operation(
            clone!(@weak self as this, @strong task_uuid => move |transaction, operation, progress| {
                let task_step = TaskStep::new_flatpak(
                    transaction,
                    operation,
                    Some(progress),
                    false
                );
                let response = Response::new_update(task_uuid.to_string(), task_step);
                this.sender.try_send(response).unwrap();

                progress.set_update_frequency(500);
                progress.connect_changed(
                    clone!(@weak this, @strong task_uuid, @weak transaction, @weak operation => move |progress|{
                        let task_step = TaskStep::new_flatpak(
                            &transaction,
                            &operation,
                            Some(progress),
                            false,
                        );
                        let response = Response::new_update(task_uuid.to_string(), task_step);
                        this.sender.try_send(response).unwrap();
                    }),
                );
            }),
        );

        transaction.connect_operation_done(
            clone!(@weak self as this, @strong task_uuid => move |transaction, operation, _, _| {
                let task_step = TaskStep::new_flatpak(
                            transaction,
                            operation,
                            None,
                            true,
                        );
                let response = Response::new_update(task_uuid.to_string(), task_step);
                this.sender.try_send(response).unwrap();

                // Check if this was the last operation ("step") -> whole task is done
                let index = transaction
                    .operations()
                    .iter()
                    .position(|o| o == operation)
                    .unwrap();
                if index +1 == transaction.operations().len() {
                    let result = TaskResult::new_done();
                    let response = Response::new_result(task_uuid.to_string(), result);
                    this.sender.try_send(response).unwrap();
                }
            }),
        );

        let cancellable = gio::Cancellable::new();
        // Own scope so that the mutex gets unlocked again
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(task_uuid.clone(), cancellable.clone());
        }

        // Start the actual Flatpak transaction
        // This is going to block the thread till completion
        transaction.run(Some(&cancellable))?;

        let mut transactions = self.transactions.lock().unwrap();
        transactions.remove(&task_uuid);

        Ok(())
    }

    fn run_dry_run_transaction(
        &self,
        transaction: Transaction,
        // We need the "real" installation (not the dry-run one) to check what's already installed
        installation_info: &InstallationInfo,
        // Whether to load the AppStream data from the Remote
        load_remote_appstream: bool,
    ) -> Result<DryRunResult, WorkerError> {
        let dry_run_result = Rc::new(RefCell::new(DryRunResult::default()));
        let real_installation = Installation::from(installation_info);

        // Check if new remotes are added during the transaction
        let installation_info = installation_info.clone();
        transaction.connect_add_new_remote(
            clone!(@weak dry_run_result, @strong installation_info => @default-return false, move |_, _, _, name, url|{
                dry_run_result.borrow_mut().remotes.push((name.into(), url.into()));
                true
            }),
        );

        // Ready -> Everything got resolved, so we can check the transaction operations
        transaction.connect_ready(
            clone!(@weak dry_run_result, @weak real_installation, @strong load_remote_appstream => @default-return false, move |transaction|{
                let operation_count = transaction.operations().len();
                for (pos, operation) in transaction.operations().iter().enumerate () {
                    // Check if it's the last operation, which is the actual app / runtime
                    if (pos+1) ==  operation_count {
                        let operation_commit = operation.commit().unwrap().to_string();
                        let operation_metadata = operation.metadata().unwrap().to_data().to_string();
                        let operation_old_metadata = operation.metadata().map(|m| m.to_data().to_string());

                        dry_run_result.borrow_mut().ref_ = operation.get_ref().unwrap().to_string();
                        dry_run_result.borrow_mut().commit = operation_commit.clone();
                        dry_run_result.borrow_mut().metainfo = operation_metadata;
                        dry_run_result.borrow_mut().old_metainfo = operation_old_metadata.into();
                        dry_run_result.borrow_mut().download_size = operation.download_size();
                        dry_run_result.borrow_mut().installed_size = operation.installed_size();

                        if operation.operation_type() == TransactionOperationType::InstallBundle{
                            dry_run_result.borrow_mut().has_update_source = false;
                        }

                        // Check if ref is already installed
                        let ref_ = Ref::parse(&operation.get_ref().unwrap()).unwrap();
                        let installed = real_installation.installed_ref(
                            ref_.kind(),
                            &ref_.name().unwrap(),
                            Some(&ref_.arch().unwrap()),
                            Some(&ref_.branch().unwrap()),
                            Cancellable::NONE
                        );

                        if let Ok(installed) = installed {
                            let installed_origin = installed.origin().unwrap();
                            let operation_remote = operation.remote().unwrap();

                            // Check if the ref is already installed, but from a different remote
                            // If yes, it the already installed ref needs to get uninstalled first,
                            // before the new one can get installed.
                            if installed_origin != operation_remote {
                                dry_run_result.borrow_mut().is_replacing_remote = Some(installed_origin.to_string()).into();
                            }

                            if installed.commit().unwrap() == operation_commit {
                                // Commit is the same -> ref is already installed
                                dry_run_result.borrow_mut().is_already_installed = true;
                            }else{
                                // Commit differs -> is update
                                dry_run_result.borrow_mut().is_update = true;
                            }
                        }

                        // Load appstream metadata
                        if load_remote_appstream {
                            let dry_run_installation = transaction.installation().unwrap();
                            let remote_name = operation.remote().unwrap().to_string();
                            let arch = ref_.arch().unwrap().to_string();

                            // Check if remote is already added (and we don't need to update the appstram data)
                            let remote = match real_installation.remote_by_name(&remote_name, Cancellable::NONE){
                                Ok(remote) => remote,
                                Err(_) => {
                                    debug!("Update appstream data for remote \"{}\" in dry run installation...", remote_name);
                                    let res = dry_run_installation.update_appstream_sync(&remote_name, Some(&arch), Cancellable::NONE);
                                    if let Err(err) = res {
                                        warn!("Unable to update appstream data: {}", err.to_string());
                                        return false;
                                    }

                                    dry_run_installation.remote_by_name(&remote_name, Cancellable::NONE).unwrap()
                                }
                            };

                            if let Some(component) = appstream::utils::component_from_remote(&ref_, &remote){
                                // Appstream
                                let json = serde_json::to_string(&component).unwrap();
                                dry_run_result.borrow_mut().appstream_component = Some(json).into();

                                // Icon
                                let appstream_dir = remote.appstream_dir(Some(&arch)).unwrap();
                                let icon = appstream_dir.child(format!("icons/128x128/{}.png", ref_.name().unwrap()));
                                if let Ok((bytes, _)) = icon.load_bytes(Cancellable::NONE){
                                    dry_run_result.borrow_mut().icon = bytes.to_vec();
                                }
                            }else{
                                warn!("Couldn't find appstream component.");
                            }
                        }
                    }else{
                        let runtime = DryRunRuntime{
                            ref_: operation.get_ref().unwrap().to_string(),
                            operation_type: operation.operation_type().to_str().unwrap().to_string(),
                            download_size: operation.download_size(),
                            installed_size: operation.installed_size(),
                        };
                        dry_run_result.borrow_mut().runtimes.push(runtime);
                    }
                }

                // Do not allow the install to start, since it's a dry run
                false
            }),
        );

        if let Err(err) = transaction.run(Cancellable::NONE) {
            if err.kind::<flatpak::Error>() == Some(flatpak::Error::RuntimeNotFound) {
                // Unfortunately there's no clean way to find out which runtime is missing
                // so we have to parse the error message to find the runtime ref.
                let regex = regex::Regex::new(r".+ (.+/.+/[^ ]+) .+ (.+/.+/[^ ]+) .+").unwrap();
                let error = if let Some(runtimes) = regex.captures(err.message()) {
                    let runtime = runtimes[2].parse().unwrap();
                    WorkerError::DryRunRuntimeNotFound(runtime)
                } else {
                    WorkerError::DryRunRuntimeNotFound("unknown-runtime".to_string())
                };

                return Err(error);
            } else if err.kind::<flatpak::Error>() != Some(flatpak::Error::Aborted) {
                error!("Error during transaction dry run: {}", err.message());
                return Err(err.into());
            }
        }

        let results = dry_run_result.borrow().clone();
        Ok(results)
    }

    fn new_transaction(
        &self,
        installation_info: &InstallationInfo,
        dry_run: bool,
    ) -> Result<Transaction, WorkerError> {
        let installation = Installation::from(installation_info);

        // Setup a own installation for dry run transactions, and add the specified
        // installation as dependency source. This way the dry run transaction
        // doesn't touch the specified installation, but has nevertheless the same local
        // runtimes available.

        // TODO: There's a race condition when you run multiple dry-run transactions at
        // the same time, since they use the same installation
        let transaction = if dry_run {
            let remotes = installation.list_remotes(Cancellable::NONE)?;
            let mut dry_run_path = glib::tmp_dir();
            dry_run_path.push("souk-dry-run");

            // Remove previous dry run installation
            let _ = std::fs::remove_dir_all(&dry_run_path);

            // New temporary dry run installation
            std::fs::create_dir_all(&dry_run_path).expect("Unable to create dry run installation");
            let file = gio::File::for_path(dry_run_path);
            let dry_run = Installation::for_path(&file, true, Cancellable::NONE)?;

            // Add the same remotes to the dry run installation
            for remote in remotes {
                if remote.url().unwrap().is_empty() || remote.is_disabled() {
                    debug!(
                        "Skip remote {} for dry run installation, no url or disabled.",
                        remote.name().unwrap()
                    );
                    continue;
                }

                // For whatever reason we have to create a new remote object
                let remote_to_add = Remote::new(&remote.name().unwrap());
                remote_to_add.set_url(&remote.url().unwrap());
                // We can't retrieve the public key of the added remote, so we trust that the
                // the added remotes are valid, and disable gpg verify for the dry run
                // transaction
                remote_to_add.set_gpg_verify(false);
                dry_run.add_remote(&remote_to_add, true, Cancellable::NONE)?;
            }

            // Create new transaction, and add the "real" installation as dependency source
            let t = Transaction::for_installation(&dry_run, Cancellable::NONE)?;
            t.add_dependency_source(&installation);

            t
        } else {
            Transaction::for_installation(&installation, Cancellable::NONE)?
        };

        // Add all default installations (= system wide installations) as dependency
        // source
        transaction.add_default_dependency_sources();

        Ok(transaction)
    }

    fn uninstall_ref(
        &self,
        ref_: &str,
        installation_info: &InstallationInfo,
    ) -> Result<(), WorkerError> {
        debug!("Uninstall: {}", ref_);

        let transaction = self.new_transaction(installation_info, false)?;
        transaction.add_uninstall(ref_)?;
        transaction.run(Cancellable::NONE)?;
        Ok(())
    }

    /// Downloads the .flatpakrepo file for a remote
    fn retrieve_flatpak_remote(&self, repo_url: &str) -> Result<Remote, WorkerError> {
        let mut response = isahc::get(repo_url)?;
        let bytes = glib::Bytes::from_owned(response.bytes()?);

        Ok(Remote::from_file("remote", &bytes)?)
    }
}
