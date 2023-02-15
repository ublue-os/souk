// Souk - worker.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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
use std::path::PathBuf;
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

use crate::shared::flatpak::dry_run::{DryRun, DryRunPackage};
use crate::shared::flatpak::info::{InstallationInfo, PackageInfo, RemoteInfo};
use crate::shared::flatpak::FlatpakOperationKind;
use crate::shared::task::{FlatpakTask, FlatpakTaskKind, TaskProgress, TaskResponse, TaskResult};
use crate::shared::WorkerError;
use crate::worker::appstream;

#[derive(Debug, Clone, Downgrade)]
pub struct FlatpakWorker {
    transactions: Arc<Mutex<HashMap<String, Cancellable>>>,
    sender: Arc<Sender<TaskResponse>>,
}

impl FlatpakWorker {
    pub fn new(sender: Sender<TaskResponse>) -> Self {
        Self {
            transactions: Arc::default(),
            sender: Arc::new(sender),
        }
    }

    pub fn process_task(&self, task: FlatpakTask, task_uuid: &str) {
        let result = match task.kind {
            FlatpakTaskKind::Install => {
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
            FlatpakTaskKind::InstallBundleFile => {
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
            FlatpakTaskKind::InstallRefFile => {
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
            FlatpakTaskKind::Update => {
                unimplemented!();
            }
            FlatpakTaskKind::UpdateInstallation => {
                unimplemented!();
            }
            FlatpakTaskKind::Uninstall => {
                unimplemented!();
            }
            FlatpakTaskKind::None => return,
        };

        if let Err(err) = result {
            // Transaction got cancelled (probably by user)
            if err == WorkerError::GLibCancelled(String::new()) {
                let result = TaskResult::new_cancelled();
                let response = TaskResponse::new_result(task_uuid.into(), result);
                self.sender.try_send(response).unwrap();
            } else {
                let result = TaskResult::new_error(err);
                let response = TaskResponse::new_result(task_uuid.into(), result);
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

        let transaction = self.new_transaction(task_uuid, installation_info, false)?;

        if uninstall_before_install {
            self.uninstall_ref(task_uuid, ref_, installation_info)?;
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

        let transaction = self.new_transaction(task_uuid, installation_info, false)?;

        if uninstall_before_install {
            let bundle = BundleRef::new(&file)?;
            let ref_ = bundle.format_ref().unwrap();
            self.uninstall_ref(task_uuid, &ref_, installation_info)?;
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

        let transaction = self.new_transaction(task_uuid, installation_info, true)?;
        transaction.add_install_bundle(&file, None)?;

        let mut results =
            self.run_dry_run_transaction(task_uuid, transaction, installation_info)?;

        // Installed bundle size
        results.package.installed_size = bundle.installed_size();

        // Remotes
        if let Some(runtime_repo_url) = bundle.runtime_repo_url() {
            // Download Flatpak repofile for additional remote metadata
            let bundle_remote = self.retrieve_flatpak_remote(&runtime_repo_url)?;

            for remote_info in &mut results.remotes {
                if bundle_remote.url().unwrap().as_str() == remote_info.repository_url {
                    remote_info.update_metadata(&bundle_remote);
                    break;
                }
            }
        }

        // Icon
        if let Some(bytes) = bundle.icon(128) {
            results.package.icon = Some(bytes.to_vec()).into();
        }

        // Appstream
        if let Some(compressed) = bundle.appstream() {
            let collection = Collection::from_gzipped_bytes(&compressed).unwrap();
            let component = &collection.components[0];

            let json = serde_json::to_string(component).unwrap();
            results.package.appstream_component = Some(json).into();
        }

        let result = TaskResult::new_dry_run(results);
        let response = TaskResponse::new_result(task_uuid.to_string(), result);
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

        let transaction = self.new_transaction(task_uuid, installation_info, false)?;

        if uninstall_before_install {
            let keyfile = KeyFile::new();
            keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;
            let name = keyfile.value("Flatpak Ref", "Name")?;
            let branch = keyfile.value("Flatpak Ref", "Branch")?;
            let arch = flatpak::functions::default_arch().unwrap();

            let ref_ = format!("app/{name}/{arch}/{branch}");
            self.uninstall_ref(task_uuid, &ref_, installation_info)?;
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

        let transaction = self.new_transaction(task_uuid, installation_info, true)?;
        transaction.add_install_flatpakref(&bytes)?;

        let mut results =
            self.run_dry_run_transaction(task_uuid, transaction, installation_info)?;

        // Up to two remotes can be added during a *.flatpakref installation:
        // 1) `Url` value (= the repository where the ref is located)
        // 2) `RuntimeRepo` value (doesn't need to point to the same repo as
        // `Url`)

        let keyfile = KeyFile::new();
        keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;

        if let Ok(repo_url) = keyfile.value("Flatpak Ref", "RuntimeRepo") {
            if !repo_url.is_empty() {
                let ref_repo = self.retrieve_flatpak_remote(&repo_url)?;
                let ref_repo_url = ref_repo.url().unwrap().to_string();

                for remote_info in &mut results.remotes {
                    if ref_repo_url == remote_info.repository_url {
                        remote_info.update_metadata(&ref_repo);
                        break;
                    }
                }
            }
        }

        let result = TaskResult::new_dry_run(results);
        let response = TaskResponse::new_result(task_uuid.to_string(), result);
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
                let mut task_operations = Vec::new();
                for operation in transaction.operations(){
                    let task_operation = TaskProgress::new_flatpak(transaction, &operation, None, false);
                    task_operations.push(task_operation);
                }

                let response = TaskResponse::new_initial(task_uuid.clone(), task_operations);
                sender.try_send(response).unwrap();

                // Real transaction -> start (unlike dryrun)
                true
            }),
        );

        transaction.connect_new_operation(
            clone!(@weak self as this, @strong task_uuid => move |transaction, operation, progress| {
                let task_progress = TaskProgress::new_flatpak(
                    transaction,
                    operation,
                    Some(progress),
                    false
                );
                let response = TaskResponse::new_update(task_uuid.to_string(), task_progress);
                this.sender.try_send(response).unwrap();

                progress.set_update_frequency(500);
                progress.connect_changed(
                    clone!(@weak this, @strong task_uuid, @weak transaction, @weak operation => move |progress|{
                        let task_progress = TaskProgress::new_flatpak(
                            &transaction,
                            &operation,
                            Some(progress),
                            false,
                        );
                        let response = TaskResponse::new_update(task_uuid.to_string(), task_progress);
                        this.sender.try_send(response).unwrap();
                    }),
                );
            }),
        );

        transaction.connect_operation_done(
            clone!(@weak self as this, @strong task_uuid => move |transaction, operation, _, _| {
                let task_progress = TaskProgress::new_flatpak(
                            transaction,
                            operation,
                            None,
                            true,
                        );
                let response = TaskResponse::new_update(task_uuid.to_string(), task_progress);
                this.sender.try_send(response).unwrap();

                // Check if this was the last operation ("step") -> whole task is done
                let index = transaction
                    .operations()
                    .iter()
                    .position(|o| o == operation)
                    .unwrap();
                if index +1 == transaction.operations().len() {
                    let result = TaskResult::new_done();
                    let response = TaskResponse::new_result(task_uuid.to_string(), result);
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
        task_uuid: &str,
        transaction: Transaction,
        // We need the "real" installation (not the dry-run one) to check what's already installed
        installation_info: &InstallationInfo,
    ) -> Result<DryRun, WorkerError> {
        let result = Rc::new(RefCell::new(DryRun::default()));
        let real_installation = Installation::from(installation_info);

        // Check if new remotes are added during the transaction
        let installation_info = installation_info.clone();
        transaction.connect_add_new_remote(
            clone!(@weak result, @strong installation_info => @default-return false, move |_, _, _, name, url|{
                let remote_info = RemoteInfo::new(name.into(), url.into(), None);
                result.borrow_mut().remotes.push(remote_info);
                true
            }),
        );

        // Ready -> Everything got resolved, so we can check the transaction operations
        transaction.connect_ready(
            clone!(@weak result, @weak real_installation => @default-return false, move |transaction|{
                let mut result = result.borrow_mut();
                let operation_count = transaction.operations().len();

                for (pos, operation) in transaction.operations().iter().enumerate () {
                    let operation_ref = operation.get_ref().unwrap().to_string();
                    let operation_commit = operation.commit().unwrap().to_string();
                    let operation_metadata = operation.metadata().unwrap().to_data().to_string();
                    let operation_old_metadata = operation.metadata().map(|m| m.to_data().to_string());

                    // Retrieve remote
                    let remote_name = operation.remote().unwrap().to_string();
                    let remote_info = if let Ok(f_remote) = real_installation.remote_by_name(&remote_name, Cancellable::NONE){
                        RemoteInfo::from_flatpak(&f_remote, Some(&real_installation))
                    } else {
                        // Remote doesn't exist in real installation
                        let f_remote = transaction.installation().unwrap().remote_by_name(&remote_name, Cancellable::NONE).unwrap();
                        RemoteInfo::from_flatpak(&f_remote, None)
                    };

                    // Load appstream
                    let mut appstream = None;
                    let mut icon = None;

                    // We can't load appstream for bundles from the remote, since it's included in the bundle file
                    if operation.operation_type() != TransactionOperationType::InstallBundle {
                        let dry_run_installation = transaction.installation().unwrap();
                        let remote_name = operation.remote().unwrap().to_string();
                        let ref_ = Ref::parse(&operation.get_ref().unwrap()).unwrap();
                        let ref_name = ref_.name().unwrap().to_string();
                        let arch = ref_.arch().unwrap().to_string();

                        // Those Flatpak subrefs usually don't include appstream data.
                        // So we strip the suffixes, and retrieve the appstream data of the actual ref.
                        //
                        // We use here the same subrefs as Flatpak, see:
                        // https://github.com/flatpak/flatpak/blob/600e18567c538ecd306d021534dbb418dc490676/common/flatpak-ref-utils.c#L451
                        let ref_name = if ref_name.ends_with(".Locale")
                            || ref_name.ends_with(".Debug")
                            || ref_name.ends_with(".Sources")
                        {
                            let mut rn = ref_name.replace(".Locale", "");
                            rn = rn.replace(".Debug", "");
                            rn = rn.replace(".Sources", "");
                            rn
                        }else{
                            ref_name.to_string()
                        };

                        // Check if remote is already added (and we don't need to update the appstram data)
                        let remote = match real_installation.remote_by_name(&remote_name, Cancellable::NONE) {
                            Ok(remote) => remote,
                            Err(_) => {
                                debug!("Update appstream data for remote \"{}\" in dry run installation...", remote_name);
                                let res = dry_run_installation.update_appstream_sync(&remote_name, Some(&arch), Cancellable::NONE);
                                if let Err(err) = res {
                                    warn!("Unable to update appstream data: {}", err.to_string());
                                }

                                dry_run_installation.remote_by_name(&remote_name, Cancellable::NONE).unwrap()
                            }
                        };

                        // TODO: This currently compiles the appstream into xmlb every single time for every single runtime / package....
                        if let Some(component) = appstream::utils::component_from_remote(&ref_name, &arch, &remote) {
                            // Appstream
                            let json = serde_json::to_string(&component).unwrap();
                            appstream = Some(json);

                            // Icon
                            let appstream_dir = remote.appstream_dir(Some(&arch)).unwrap();
                            let icon_file = appstream_dir.child(format!("icons/128x128/{}.png", ref_.name().unwrap()));
                            if let Ok((bytes, _)) = icon_file.load_bytes(Cancellable::NONE) {
                                icon = Some(bytes.to_vec());
                            }
                        } else {
                            warn!("Couldn't find appstream component for {operation_ref}");
                        }
                    }

                    // Check if it's the last operation, which is the targeted app / runtime
                    if (pos+1) ==  operation_count {
                        // Package
                        let package = DryRunPackage{
                            info: PackageInfo::new(operation_ref, remote_info),
                            operation_kind: operation.operation_type().into(),
                            download_size: operation.download_size(),
                            installed_size: operation.installed_size(),
                            icon: icon.into(),
                            appstream_component: appstream.into(),
                            metadata: operation_metadata,
                            old_metadata: operation_old_metadata.into(),
                        };
                        result.package = package;

                        if operation.operation_type() == TransactionOperationType::InstallBundle{
                            result.has_update_source = false;
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
                                let f_remote = real_installation.remote_by_name(&installed_origin, Cancellable::NONE).unwrap();
                                let remote_info = RemoteInfo::from_flatpak(&f_remote, Some(&real_installation));

                                result.is_replacing_remote = Some(remote_info).into();
                            }

                            if installed.commit().unwrap() == operation_commit {
                                // Commit is the same -> ref is already installed -> No operation
                                result.package.operation_kind = FlatpakOperationKind::None;
                            }else{
                                // Commit differs -> is update
                                // Manually set operation type to `Update`, since technically it's
                                // not possible to "update" Flatpak bundles, from libflatpak side
                                // it's always `InstallBundle` operation.
                                result.package.operation_kind = FlatpakOperationKind::Update;
                            }
                        }
                    }else{
                        let runtime = DryRunPackage{
                            info: PackageInfo::new(operation_ref, remote_info),
                            operation_kind: operation.operation_type().into(),
                            download_size: operation.download_size(),
                            installed_size: operation.installed_size(),
                            icon: icon.into(),
                            appstream_component: appstream.into(),
                            metadata: operation_metadata,
                            old_metadata: operation_old_metadata.into(),
                        };

                        result.runtimes.push(runtime);
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

                Self::cleanup_dry_run_installation(task_uuid);
                return Err(error);
            } else if err.kind::<flatpak::Error>() != Some(flatpak::Error::Aborted) {
                error!("Error during transaction dry run: {}", err.message());

                Self::cleanup_dry_run_installation(task_uuid);
                return Err(err.into());
            }
        }

        // Remove temporary dry run installation directory again
        Self::cleanup_dry_run_installation(task_uuid);

        let result = result.borrow().clone();
        Ok(result)
    }

    fn new_transaction(
        &self,
        task_uuid: &str,
        installation_info: &InstallationInfo,
        dry_run: bool,
    ) -> Result<Transaction, WorkerError> {
        let installation = Installation::from(installation_info);

        // Setup a own installation for dry run transactions, and add the specified
        // installation as dependency source. This way the dry run transaction
        // doesn't touch the specified installation, but has nevertheless the same local
        // runtimes available.
        let transaction = if dry_run {
            let mut path = glib::tmp_dir();
            path.push(format!("souk-dry-run-{}", task_uuid));

            let dry_run_installation = match Self::dry_run_installation(&path, &installation) {
                Ok(i) => i,
                Err(err) => {
                    error!("Unable to setup dry run installation: {}", err.to_string());
                    Self::cleanup_dry_run_installation(task_uuid);
                    return Err(err);
                }
            };

            // Create new transaction, and add the "real" installation as dependency source
            let t = Transaction::for_installation(&dry_run_installation, Cancellable::NONE)?;
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

    fn dry_run_installation(
        path: &PathBuf,
        real_installation: &Installation,
    ) -> Result<Installation, WorkerError> {
        // New temporary dry run installation
        std::fs::create_dir_all(&path).expect("Unable to create dry run installation");
        let file = gio::File::for_path(path);

        let dry_run_installation = Installation::for_path(&file, true, Cancellable::NONE)?;

        // Add the same remotes to the dry run installation
        let remotes = real_installation.list_remotes(Cancellable::NONE)?;
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
            dry_run_installation.add_remote(&remote_to_add, false, Cancellable::NONE)?;
        }

        Ok(dry_run_installation)
    }

    fn cleanup_dry_run_installation(task_uuid: &str) {
        let mut path = glib::tmp_dir();
        path.push(format!("souk-dry-run-{}", task_uuid));

        if let Err(err) = std::fs::remove_dir_all(&path) {
            warn!(
                "Unable to remove dry run installation directory: {}",
                err.to_string()
            );
        }
    }

    fn uninstall_ref(
        &self,
        task_uuid: &str,
        ref_: &str,
        installation_info: &InstallationInfo,
    ) -> Result<(), WorkerError> {
        debug!("Uninstall: {}", ref_);

        let transaction = self.new_transaction(task_uuid, installation_info, false)?;
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
