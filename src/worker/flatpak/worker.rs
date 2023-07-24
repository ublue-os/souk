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
use crate::shared::flatpak::info::RemoteInfo;
use crate::shared::flatpak::FlatpakOperationKind;
use crate::shared::task::response::{OperationActivity, TaskResponse, TaskResult};
use crate::shared::task::{FlatpakTask, FlatpakTaskKind};
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

    // TODO: Check if this can be further simplified
    pub fn process_task(&self, task: FlatpakTask) {
        let result = match task.kind {
            FlatpakTaskKind::Install => {
                if task.dry_run {
                    unimplemented!();
                } else {
                    self.install_flatpak(&task)
                }
            }
            FlatpakTaskKind::InstallBundleFile => {
                if task.dry_run {
                    self.install_flatpak_bundle_file_dry_run(&task)
                } else {
                    self.install_flatpak_bundle_file(&task)
                }
            }
            FlatpakTaskKind::InstallRefFile => {
                if task.dry_run {
                    self.install_flatpak_ref_file_dry_run(&task)
                } else {
                    self.install_flatpak_ref_file(&task)
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
                let result = TaskResult::Cancelled;
                let response = TaskResponse::new_result(task.into(), result);
                self.sender.try_send(response).unwrap();
            } else {
                let result = TaskResult::Error(Box::new(err));
                let response = TaskResponse::new_result(task.into(), result);
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

    fn install_flatpak(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let ref_ = task.ref_.as_ref().unwrap();
        let remote = task.remote.as_ref().unwrap();
        info!("Install Flatpak: {}", ref_);

        if task.uninstall_before_install {
            self.uninstall_before_install(task, ref_)?;
        }

        let transaction = self.new_transaction(task)?;
        transaction.add_install(&remote.name, ref_, &[])?;
        self.run_transaction(task, transaction, false)?;

        Ok(())
    }

    fn install_flatpak_bundle_file(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let path = task.path.as_ref().unwrap();
        let file = gio::File::for_parse_name(path);
        info!("Install Flatpak bundle: {}", path);

        if task.uninstall_before_install {
            let bundle = BundleRef::new(&file)?;
            let ref_ = bundle.format_ref().unwrap();
            self.uninstall_before_install(task, &ref_)?;
        }

        let transaction = self.new_transaction(task)?;
        transaction.add_install_bundle(&file, None)?;
        self.run_transaction(task, transaction, false)?;

        Ok(())
    }

    fn install_flatpak_bundle_file_dry_run(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let path = task.path.as_ref().unwrap();
        let file = gio::File::for_parse_name(path);
        let bundle = BundleRef::new(&file)?;
        info!("Install Flatpak bundle (dry run): {}", path);

        // Run the transaction as dry run
        let transaction = self.new_transaction(task)?;
        transaction.add_install_bundle(&file, None)?;
        let mut res = self.run_dry_run_transaction(task, transaction)?;

        // Check if bundle has a source for updates
        res.has_update_source = bundle.origin().is_some();

        // Installed bundle size
        res.package.installed_size = bundle.installed_size();

        // Remotes
        if let Some(runtime_repo_url) = bundle.runtime_repo_url() {
            // Download Flatpak repofile for additional remote metadata
            let repo_bytes = self.retrieve_flatpak_remote(&runtime_repo_url)?;
            let bundle_remote = Remote::from_file("remote", &repo_bytes)?;

            for remote_info in &mut res.remotes {
                if bundle_remote.url().unwrap().as_str() == remote_info.repository_url {
                    remote_info.set_repo_bytes(repo_bytes.to_vec());
                    break;
                }
            }
        }

        // Icon
        if let Some(bytes) = bundle.icon(128) {
            res.package.icon = Some(bytes.to_vec());
        }

        // Appstream
        if let Some(compressed) = bundle.appstream() {
            let collection = Collection::from_gzipped_bytes(&compressed).unwrap();
            let component = &collection.components[0];

            let json = serde_json::to_string(component).unwrap();
            res.package.appstream_component = Some(json);
        }

        let result = TaskResult::DoneDryRun(Box::new(res));
        let response = TaskResponse::new_result(task.clone().into(), result);
        self.sender.try_send(response).unwrap();

        Ok(())
    }

    fn install_flatpak_ref_file(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let path = task.path.as_ref().unwrap();
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;
        info!("Install Flatpak ref: {}", path);

        if task.uninstall_before_install {
            let keyfile = KeyFile::new();
            keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;

            let ref_ = Self::parse_ref_file(&keyfile)?;
            self.uninstall_before_install(task, &ref_)?;
        }

        let transaction = self.new_transaction(task)?;
        transaction.add_install_flatpakref(&bytes)?;
        self.run_transaction(task, transaction, false)?;

        Ok(())
    }

    fn install_flatpak_ref_file_dry_run(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let path = task.path.as_ref().unwrap();
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;
        info!("Install Flatpak ref (dry run): {}", path);

        // Run the transaction as dry run
        let transaction = self.new_transaction(task)?;
        transaction.add_install_flatpakref(&bytes)?;
        let mut res = self.run_dry_run_transaction(task, transaction)?;

        // Up to two remotes can be added during a *.flatpakref installation:
        // 1) `Url` value (= the repository where the ref is located)
        // 2) `RuntimeRepo` value (doesn't need to point to the same repo as
        // `Url`)
        let keyfile = KeyFile::new();
        keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;

        if let Ok(repo_url) = keyfile.value("Flatpak Ref", "RuntimeRepo") {
            if !repo_url.is_empty() {
                let repo_bytes = self.retrieve_flatpak_remote(&repo_url)?;
                let ref_repo = Remote::from_file("remote", &repo_bytes)?;
                let ref_repo_url = ref_repo.url().unwrap().to_string();

                for remote_info in &mut res.remotes {
                    if ref_repo_url == remote_info.repository_url {
                        remote_info.set_repo_bytes(repo_bytes.to_vec());
                        break;
                    }
                }
            }
        }

        let result = TaskResult::DoneDryRun(Box::new(res));
        let response = TaskResponse::new_result(task.clone().into(), result);
        self.sender.try_send(response).unwrap();

        Ok(())
    }

    #[allow(dead_code)]
    fn uninstall_flatpak(&self, task: &FlatpakTask) -> Result<(), WorkerError> {
        let ref_ = task.ref_.as_ref().unwrap();
        info!("Uninstall Flatpak: {}", ref_);

        let transaction = self.new_transaction(task)?;
        transaction.add_uninstall(ref_)?;
        self.run_transaction(task, transaction, false)?;

        Ok(())
    }

    /// If `skip_task_result` is set, no [TaskResult::Done] gets emitted.
    /// Required if the Flatpak transaction is only part of a task and therefore
    /// does not complete it.
    fn run_transaction(
        &self,
        task: &FlatpakTask,
        transaction: Transaction,
        skip_task_result: bool,
    ) -> Result<(), WorkerError> {
        transaction.connect_add_new_remote(move |_, _, _, _, _| true);

        transaction.connect_ready(
            clone!(@strong task, @weak self.sender as sender => @default-return true, move |transaction|{
                if transaction.operations().is_empty(){
                    warn!("Transaction has no operations.");
                    return true;
                }

                let mut operation_activities = Vec::new();
                for op in transaction.operations() {
                    operation_activities.push(OperationActivity::from_flatpak_operation(transaction, &op, None, false));
                }

                let response = TaskResponse::new_operation_activity(task.clone().into(), operation_activities);
                sender.try_send(response).unwrap();

                // Real transaction -> start (unlike dryrun)
                true
            }),
        );

        transaction.connect_new_operation(
            clone!(@strong task, @weak self.sender as sender => move |transaction, operation, progress| {
                let operation_activity = OperationActivity::from_flatpak_operation(
                    transaction,
                    operation,
                    Some(progress),
                    false
                );
                let response = TaskResponse::new_operation_activity(task.clone().into(), vec![operation_activity]);
                sender.try_send(response).unwrap();

                progress.set_update_frequency(750);
                progress.connect_changed(
                    clone!(@strong task, @weak sender, @weak transaction, @weak operation => move |progress|{
                        let operation_activity = OperationActivity::from_flatpak_operation(
                            &transaction,
                            &operation,
                            Some(progress),
                            false,
                        );
                        let response = TaskResponse::new_operation_activity(task.clone().into(), vec![operation_activity]);
                        sender.try_send(response).unwrap();
                    }),
                );
            }),
        );

        transaction.connect_operation_done(
            clone!(@strong task, @strong skip_task_result, @weak self.sender as sender  => move |transaction, operation, _, _| {
                let operation_activity = OperationActivity::from_flatpak_operation(
                    transaction,
                    operation,
                    None,
                    true,
                );
                let response = TaskResponse::new_operation_activity(task.clone().into(), vec![operation_activity]);
                sender.try_send(response).unwrap();

                let index = transaction
                    .operations()
                    .iter()
                    .position(|o| o == operation)
                    .unwrap();

                // Check if this was the last operation -> whole task is done
                if index +1 == transaction.operations().len() && !skip_task_result{
                    let result = TaskResult::Done;
                    let response = TaskResponse::new_result(task.clone().into(), result);
                    sender.try_send(response).unwrap();
                }
            }),
        );

        let cancellable = gio::Cancellable::new();
        // Own scope so that the mutex gets unlocked again
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(task.uuid.clone(), cancellable.clone());
        }

        // Start the actual Flatpak transaction
        // This is going to block the thread till completion
        transaction.run(Some(&cancellable))?;

        let mut transactions = self.transactions.lock().unwrap();
        transactions.remove(&task.uuid);

        Ok(())
    }

    fn run_dry_run_transaction(
        &self,
        task: &FlatpakTask,
        transaction: Transaction,
    ) -> Result<DryRun, WorkerError> {
        debug!("Run dry run transaction: {}", task.uuid);

        let result: Rc<RefCell<DryRun>> = Rc::default();
        let real_installation = Installation::from(&task.installation);
        let dry_run_installation = transaction.installation().unwrap();

        // Check if new remotes are getting added during the transaction
        transaction.connect_add_new_remote(
            clone!(@weak result => @default-return false, move |_, _, _, name, url|{
                let remote_info = RemoteInfo::new(name.into(), url.into(), None);
                result.borrow_mut().remotes.push(remote_info);
                true
            }),
        );

        // Ready -> Everything got resolved.
        transaction.connect_ready_pre_auth(move |_| {
            // Do not allow the transaction to start, since it's a dry run
            false
        });

        // Run transaction
        let transaction_result = transaction.run(Cancellable::NONE);
        let mut result = result.borrow_mut();

        // Retrieve operations that would be performed during the transaction
        let ops = transaction.operations();
        let mut operations = ops.iter().peekable();

        // Iterate through the operations and set the values for `DryRun`
        while let Some(operation) = operations.next() {
            let op_ref = Ref::parse(&operation.get_ref().unwrap())?;
            let op_ref_str = operation.get_ref().unwrap().to_string();
            let op_commit = operation.commit().unwrap().to_string();
            let op_remote = operation.remote().unwrap();

            // Check if this is the last operation. This ref is the target of the Flatpak
            // transaction.
            let is_targeted_ref = operations.peek().is_none();

            // Retrieve remote
            let remote_name = operation.remote().unwrap().to_string();
            let (remote, remote_info) = match real_installation
                .remote_by_name(&remote_name, Cancellable::NONE)
            {
                Ok(remote) => {
                    let remote_info = RemoteInfo::from_flatpak(&remote, &real_installation);
                    (remote, remote_info)
                }
                Err(_) => {
                    // Remote doesn't exist in real installation
                    let r = dry_run_installation.remote_by_name(&remote_name, Cancellable::NONE)?;
                    let remote_info = RemoteInfo::new(
                        r.name().unwrap().into(),
                        r.url().unwrap_or_default().into(),
                        None,
                    );
                    (r, remote_info)
                }
            };

            // Package
            let mut package = DryRunPackage::from_flatpak_operation(operation, &remote_info);

            // Check if ref is already installed
            let installed_ref = real_installation
                .installed_ref(
                    op_ref.kind(),
                    &op_ref.name().unwrap(),
                    Some(&op_ref.arch().unwrap()),
                    Some(&op_ref.branch().unwrap()),
                    Cancellable::NONE,
                )
                .ok();

            // Check if the ref is already installed, and if so, compare the commit to
            // determine if it is an update.
            if let Some(installed_ref) = &installed_ref {
                let origin = installed_ref.origin().unwrap();

                // Check if the ref is already installed, but from a different remote
                if origin != op_remote {
                    // If yes, it then uninstall the installed ref first. This is not strictly
                    // necessary, but can prevent some common issues (eg. gpg mismatch)
                    debug!("[remote] {op_ref_str}: Already installed from different origin \"{origin}\".");
                    if is_targeted_ref {
                        let r = real_installation.remote_by_name(&origin, Cancellable::NONE)?;
                        let remote_info = RemoteInfo::from_flatpak(&r, &real_installation);
                        result.is_replacing_remote = Some(remote_info);
                    } else {
                        warn!("Non-targeted ref {op_ref_str} is already installed in {origin} instead of {op_remote}. This behaviour is undefined.");
                    }
                } else if installed_ref.commit().unwrap() == op_commit {
                    // Same commit is already installed - nothing to do!
                    debug!("[skip] {op_ref_str}: commit is already installed.");
                    package.operation_kind = FlatpakOperationKind::None;

                    // Skip this operation, except it's the targeted ref
                    if !is_targeted_ref {
                        continue;
                    }
                } else {
                    // Commit differs - Ref gets updated during transaction!
                    debug!("[update] {op_ref_str}: installed, but commit differs.");
                    package.operation_kind = FlatpakOperationKind::Update;

                    // Set `old_metadata` value from the installed ref. This is necessary, for
                    // example, to determine new permissions.
                    let utf8 = installed_ref.load_metadata(Cancellable::NONE)?.to_vec();
                    let metadata = String::from_utf8(utf8).unwrap();
                    package.old_metadata = Some(metadata);
                }
            } else if operation.operation_type() == TransactionOperationType::InstallBundle {
                debug!("[install] {op_ref_str}: is not installed.");
                package.operation_kind = FlatpakOperationKind::InstallBundle;
            } else {
                debug!("[install] {op_ref_str}: is not installed.");
                package.operation_kind = FlatpakOperationKind::Install;
            }

            // Retrieve appstream data unless it is a Flatpak bundle which includes the data
            // in the bundle file itself (and doesn't have a "real" remote)
            if operation.operation_type() != TransactionOperationType::InstallBundle {
                appstream::utils::set_dry_run_package_appstream(&mut package, &op_ref_str, &remote);
            }

            if is_targeted_ref {
                // Target ref -> Normally the application that is to be installed
                result.package = package;
            } else {
                // No -> A dependency / runtime.
                result.runtimes.push(package);
            }
        }

        if let Err(err) = transaction_result {
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

                Self::cleanup_dry_run_installation(&task.uuid);
                return Err(error);
            } else if err.kind::<flatpak::Error>() != Some(flatpak::Error::Aborted) {
                error!("Error during transaction dry run: {}", err.message());

                Self::cleanup_dry_run_installation(&task.uuid);
                return Err(err.into());
            }
        }

        // Remove temporary dry run installation directory again
        Self::cleanup_dry_run_installation(&task.uuid);

        debug!("Dry run transaction done: {}", task.uuid);
        Ok(result.clone())
    }

    fn new_transaction(&self, task: &FlatpakTask) -> Result<Transaction, WorkerError> {
        let installation = Installation::from(&task.installation);

        let transaction = if task.dry_run {
            let mut path = glib::tmp_dir();
            path.push(format!("souk-dry-run-{}", task.uuid));

            let dry_run_installation = match Self::dry_run_installation(&path, &installation) {
                Ok(i) => i,
                Err(err) => {
                    error!("Unable to setup dry run installation: {}", err.to_string());
                    Self::cleanup_dry_run_installation(&task.uuid);
                    return Err(err);
                }
            };

            Transaction::for_installation(&dry_run_installation, Cancellable::NONE)?
        } else {
            let t = Transaction::for_installation(&installation, Cancellable::NONE)?;
            // Add all default installations (= system wide installations) as dependency
            // source
            t.add_default_dependency_sources();
            t
        };

        Ok(transaction)
    }

    fn dry_run_installation(
        path: &PathBuf,
        real_installation: &Installation,
    ) -> Result<Installation, WorkerError> {
        // New temporary dry run installation
        std::fs::create_dir_all(path).expect("Unable to create dry run installation");
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
        path.push(format!("souk-dry-run-{task_uuid}"));

        if let Err(err) = std::fs::remove_dir_all(&path) {
            warn!(
                "Unable to remove dry run installation directory: {}",
                err.to_string()
            );
        }
    }

    /// Uninstalls a ref before installing as it cannot be directly
    /// replaced/upgraded (e.g. a Flatpak bundle where the remote differs)
    /// Must be run as a separate Flatpak transaction, otherwise the
    /// installation will fail ("X is already installed"). It seems that
    /// Flatpak updates which refs are installed (and which aren't) only after a
    /// transaction.
    fn uninstall_before_install(&self, task: &FlatpakTask, ref_: &str) -> Result<(), WorkerError> {
        debug!("Uninstall before install: {}", ref_);

        // Ensure that ref is set
        let mut task_cloned = task.clone();
        task_cloned.ref_ = Some(ref_.to_string());

        let transaction = self.new_transaction(&task_cloned)?;
        transaction.add_uninstall(ref_)?;
        self.run_transaction(&task_cloned, transaction, true)?;

        Ok(())
    }

    /// Downloads the .flatpakrepo file for a remote
    fn retrieve_flatpak_remote(&self, repo_url: &str) -> Result<glib::Bytes, WorkerError> {
        let mut response = isahc::get(repo_url)?;
        Ok(glib::Bytes::from_owned(response.bytes()?))
    }

    fn parse_ref_file(keyfile: &KeyFile) -> Result<String, WorkerError> {
        let kind = if let Ok(is_runtime) = keyfile.boolean("Flatpak Ref", "IsRuntime") {
            if is_runtime {
                "runtime"
            } else {
                "app"
            }
        } else {
            "app"
        };

        let name = keyfile.value("Flatpak Ref", "Name")?;
        let branch = keyfile.value("Flatpak Ref", "Branch")?;
        let arch = flatpak::functions::default_arch().unwrap();

        Ok(format!("{kind}/{name}/{arch}/{branch}"))
    }
}
