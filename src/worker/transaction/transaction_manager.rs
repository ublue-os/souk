// Souk - transaction_manager.rs
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
use std::thread;

use ::appstream::Collection;
use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use async_std::task;
use flatpak::prelude::*;
use flatpak::{
    BundleRef, Installation, Ref, Remote, Transaction, TransactionOperation,
    TransactionOperationType,
};
use gio::prelude::*;
use gio::Cancellable;
use glib::{clone, Downgrade, KeyFile};
use gtk::{gio, glib};
use isahc::ReadResponseExt;

use super::{
    DryRunResult, DryRunRuntime, TransactionCommand, TransactionMessage, TransactionProgress,
};
use crate::shared::info::{InstallationInfo, RemoteInfo};
use crate::worker::{appstream, WorkerError};

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionManager {
    transactions: Arc<Mutex<HashMap<String, Cancellable>>>,
    sender: Arc<Sender<TransactionMessage>>,
}

impl TransactionManager {
    pub fn start(sender: Sender<TransactionMessage>, receiver: Receiver<TransactionCommand>) {
        let manager = Self {
            transactions: Arc::default(),
            sender: Arc::new(sender),
        };

        thread::spawn(clone!(@strong manager, @strong receiver => move || {
            let mut receiver = receiver;
            let fut = async move {
                while let Some(command) = receiver.next().await {
                    // TODO: Don't work with raw threads here, but us a scheduler / pool or sth
                    thread::spawn(clone!(@weak manager => move || {
                        manager.process_command(command);
                    }));
                }
            };
            task::block_on(fut);
        }));
    }

    fn process_command(&self, command: TransactionCommand) {
        debug!("Process command: {:?}", command);

        let (result, transaction_uuid) = match command {
            TransactionCommand::InstallFlatpak(
                uuid,
                ref_,
                remote,
                installation_info,
                no_update,
            ) => (
                self.install_flatpak(&uuid, &ref_, &remote, &installation_info, no_update),
                uuid,
            ),
            TransactionCommand::InstallFlatpakBundle(uuid, path, installation_info, no_update) => (
                self.install_flatpak_bundle(&uuid, &path, &installation_info, no_update),
                uuid,
            ),
            TransactionCommand::InstallFlatpakBundleDryRun(path, installation_info, sender) => {
                let result = self.install_flatpak_bundle_dry_run(&path, &installation_info);
                sender.try_send(result).unwrap();
                return;
            }
            TransactionCommand::InstallFlatpakRef(uuid, path, installation_info, no_update) => (
                self.install_flatpak_ref(&uuid, &path, &installation_info, no_update),
                uuid,
            ),
            TransactionCommand::InstallFlatpakRefDryRun(path, installation_info, sender) => {
                let result = self.install_flatpak_ref_dry_run(&path, &installation_info);
                sender.try_send(result).unwrap();
                return;
            }
            TransactionCommand::CancelTransaction(uuid) => {
                let transactions = self.transactions.lock().unwrap();
                if let Some(cancellable) = transactions.get(&uuid) {
                    cancellable.cancel();
                } else {
                    warn!("Unable to cancel transaction: {}", uuid);
                }
                return;
            }
        };

        if let Err(err) = result {
            // Transaction got cancelled (probably by user)
            if err == WorkerError::GLibCancelled {
                let progress = TransactionProgress::new(transaction_uuid, None, None, None);
                let progress = progress.cancelled();

                self.sender
                    .try_send(TransactionMessage::Progress(progress))
                    .unwrap();
            } else {
                let error = super::TransactionError::new(transaction_uuid, err.message());
                self.sender
                    .try_send(TransactionMessage::Error(error))
                    .unwrap();
            }
        }
    }

    fn install_flatpak(
        &self,
        transaction_uuid: &str,
        ref_: &str,
        remote: &str,
        installation_info: &InstallationInfo,
        no_update: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak: {}", ref_);

        let transaction = self.new_transaction(installation_info, false)?;
        transaction.add_install(remote, ref_, &[])?;

        // There are situations where we can't directly upgrade an already installed
        // ref, and we have to uninstall the old version first, before we
        // install the new version. (for example installing a ref from a
        // different remote, and the gpg signature wouldn't match)
        if no_update {
            self.uninstall_ref(ref_, installation_info)?;
        }

        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle(
        &self,
        transaction_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
        no_update: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation_info, false)?;
        transaction.add_install_bundle(&file, None)?;

        // There are situations where we can't directly upgrade an already installed
        // ref, and we have to uninstall the old version first, before we
        // install the new version. (for example installing a ref from a
        // different remote, and the gpg signature wouldn't match)
        if no_update {
            let bundle = BundleRef::new(&file)?;
            let ref_ = bundle.format_ref().unwrap();
            self.uninstall_ref(&ref_, installation_info)?;
        }

        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation_info: &InstallationInfo,
    ) -> Result<DryRunResult, WorkerError> {
        info!("Install Flatpak bundle (dry run): {}", path);
        let file = gio::File::for_parse_name(path);
        let bundle = BundleRef::new(&file)?;

        let transaction = self.new_transaction(installation_info, true)?;
        transaction.add_install_bundle(&file, None)?;

        let results = self.run_dry_run_transaction(transaction, installation_info, false);

        if let Ok(mut results) = results {
            // Installed bundle size
            results.installed_size = bundle.installed_size();

            // Remotes
            let bundle_remote =
                self.retrieve_flatpak_remote(&bundle.runtime_repo_url().unwrap())?;
            let mut remotes_info = Vec::new();
            for (name, url) in &results.remotes {
                if bundle_remote.url().unwrap().as_str() == url {
                    remotes_info.push(RemoteInfo::from(&bundle_remote));
                } else {
                    remotes_info.push(RemoteInfo::new(name, url));
                }
            }
            results.remotes_info = remotes_info;

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

            return Ok(results);
        }

        results
    }

    fn install_flatpak_ref(
        &self,
        transaction_uuid: &str,
        path: &str,
        installation_info: &InstallationInfo,
        no_update: bool,
    ) -> Result<(), WorkerError> {
        info!("Install Flatpak ref: {}", path);
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;

        let transaction = self.new_transaction(installation_info, false)?;
        transaction.add_install_flatpakref(&bytes)?;

        // There are situations where we can't directly upgrade an already installed
        // ref, and we have to uninstall the old version first, before we
        // install the new version. (for example installing a ref from a
        // different remote, and the gpg signature wouldn't match)
        if no_update {
            let keyfile = KeyFile::new();
            keyfile.load_from_bytes(&bytes, glib::KeyFileFlags::NONE)?;
            let name = keyfile.value("Flatpak Ref", "Name")?;
            let branch = keyfile.value("Flatpak Ref", "Branch")?;
            let arch = flatpak::functions::default_arch().unwrap();

            let ref_ = format!("app/{}/{}/{}", name, arch, branch);
            self.uninstall_ref(&ref_, installation_info)?;
        }

        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_ref_dry_run(
        &self,
        path: &str,
        installation_info: &InstallationInfo,
    ) -> Result<DryRunResult, WorkerError> {
        info!("Install Flatpak ref (dry run): {}", path);
        let file = gio::File::for_parse_name(path);
        let bytes = file.load_bytes(Cancellable::NONE)?.0;

        let transaction = self.new_transaction(installation_info, true)?;
        transaction.add_install_flatpakref(&bytes)?;

        let results = self.run_dry_run_transaction(transaction, installation_info, true);

        // Up to two remotes can be added during a *.flatpakref installation:
        // 1) `Url` value (= the repository where the ref is located)
        // 2) `RuntimeRepo` value (doesn't need to point to the same repository as
        // `Url`)
        if let Ok(mut results) = results {
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

            return Ok(results);
        }

        results
    }

    fn run_transaction(
        &self,
        transaction_uuid: String,
        transaction: Transaction,
    ) -> Result<(), WorkerError> {
        transaction.connect_add_new_remote(move |_, _, _, _, _| true);

        transaction.connect_new_operation(
            clone!(@weak self as this, @strong transaction_uuid => move |transaction, operation, progress| {
                this.handle_operation(transaction_uuid.clone(), transaction, operation, progress);
            }),
        );

        transaction.connect_operation_done(
            clone!(@weak self as this, @strong transaction_uuid => move |transaction, operation, _commit, _result| {
                let mut progress = TransactionProgress::new(
                    transaction_uuid.clone(),
                    Some(transaction),
                    Some(operation),
                    None,
                );

                // Check if all operations are done
                if progress.operations_count == progress.current_operation{
                    progress = progress.done();
                    this.sender.try_send(TransactionMessage::Progress(progress)).unwrap();
                }else{
                    this.sender.try_send(TransactionMessage::Progress(progress)).unwrap();
                }
            }),
        );

        let cancellable = gio::Cancellable::new();
        // Own scope so that the mutex gets unlocked again
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(transaction_uuid.clone(), cancellable.clone());
        }

        // Start the actual Flatpak transaction
        // This is going to block the thread till completion
        transaction.run(Some(&cancellable))?;

        let mut transactions = self.transactions.lock().unwrap();
        transactions.remove(&transaction_uuid);

        Ok(())
    }

    fn handle_operation(
        &self,
        transaction_uuid: String,
        transaction: &Transaction,
        transaction_operation: &TransactionOperation,
        transaction_progress: &flatpak::TransactionProgress,
    ) {
        let progress = TransactionProgress::new(
            transaction_uuid,
            Some(transaction),
            Some(transaction_operation),
            Some(transaction_progress),
        );
        self.sender
            .try_send(TransactionMessage::Progress(progress.clone()))
            .unwrap();

        transaction_progress.connect_changed(
            clone!(@weak self.sender as sender, @strong progress => move |transaction_progress|{
                let updated = progress.update(transaction_progress);
                sender.try_send(TransactionMessage::Progress(updated)).unwrap();
            }),
        );
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
