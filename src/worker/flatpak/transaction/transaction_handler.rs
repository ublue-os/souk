// Souk - transaction_handler.rs
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

use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use async_std::task;
use gio::prelude::*;
use gio::{Cancellable, IOErrorEnum};
use glib::{clone, Downgrade, Error};
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{
    BundleRef, Installation, Ref, Remote, Transaction, TransactionOperation,
    TransactionRemoteReason,
};

use super::{
    DryRunError, DryRunRemote, DryRunResults, DryRunRuntime, TransactionCommand,
    TransactionMessage, TransactionProgress,
};
use crate::worker::flatpak::installation::InstallationManager;

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionHandler {
    transactions: Arc<Mutex<HashMap<String, Cancellable>>>,
    installation_manager: Arc<InstallationManager>,
    sender: Arc<Sender<TransactionMessage>>,
}

impl TransactionHandler {
    pub fn start(
        installation_manager: InstallationManager,
        sender: Sender<TransactionMessage>,
        receiver: Receiver<TransactionCommand>,
    ) {
        let installation_manager = Arc::new(installation_manager);

        let handler = Self {
            transactions: Arc::default(),
            installation_manager,
            sender: Arc::new(sender),
        };

        thread::spawn(clone!(@strong handler, @strong receiver => move || {
            let mut receiver = receiver;
            let fut = async move {
                while let Some(command) = receiver.next().await {
                    // TODO: Don't work with raw threads here, but us a scheduler / pool or sth
                    thread::spawn(clone!(@weak handler => move || {
                        handler.process_command(command);
                    }));
                }
            };
            task::block_on(fut);
        }));
    }

    fn process_command(&self, command: TransactionCommand) {
        debug!("Process command: {:?}", command);
        let mut dry_run_sender = None;

        let (result, transaction_uuid) = match command {
            TransactionCommand::InstallFlatpak(uuid, ref_, remote, installation_uuid) => (
                self.install_flatpak(&uuid, &ref_, &remote, &installation_uuid),
                uuid,
            ),
            TransactionCommand::InstallFlatpakBundle(uuid, path, installation_uuid) => (
                self.install_flatpak_bundle(&uuid, &path, &installation_uuid),
                uuid,
            ),
            TransactionCommand::InstallFlatpakBundleDryRun(path, installation_uuid, sender) => {
                dry_run_sender = Some(sender.clone());
                (
                    self.install_flatpak_bundle_dry_run(&path, &installation_uuid, sender),
                    String::new(), // We don't have an uuid for dry runs
                )
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
            if let Some(dry_run_sender) = dry_run_sender {
                let message = err.message().to_string();
                let dry_run_error = DryRunError::Other(message);
                dry_run_sender.try_send(Err(dry_run_error)).unwrap();
                return;
            }

            // No uuid -> dry run transaction -> we don't care about errors
            if transaction_uuid.is_empty() {
                error!("Error during transaction dry run: {}", err.message());
                return;
            }

            if err.kind::<IOErrorEnum>() == Some(IOErrorEnum::Cancelled) {
                let progress = TransactionProgress::new(transaction_uuid, None, None, None);
                let progress = progress.cancelled();
                self.sender
                    .try_send(TransactionMessage::Progress(progress))
                    .unwrap();
            } else {
                let error =
                    super::TransactionError::new(transaction_uuid, err.message().to_string());
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
        installation_uuid: &str,
    ) -> Result<(), Error> {
        info!("Install Flatpak: {}", ref_);

        let transaction = self.new_transaction(installation_uuid, false);
        transaction.add_install(remote, ref_, &[])?;
        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle(
        &self,
        transaction_uuid: &str,
        path: &str,
        installation_uuid: &str,
    ) -> Result<(), Error> {
        info!("Install Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation_uuid, false);
        transaction.add_install_bundle(&file, None)?;
        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation_uuid: &str,
        sender: Sender<Result<DryRunResults, DryRunError>>,
    ) -> Result<(), Error> {
        info!("Install Flatpak bundle (dry run): {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation_uuid, true);
        transaction.add_install_bundle(&file, None)?;

        let bundle = BundleRef::new(&file)?;
        let ref_ = bundle.clone().upcast::<Ref>();

        let installation = self
            .installation_manager
            .flatpak_installation_by_uuid(installation_uuid);
        let results = self.run_dry_run_transaction(transaction, &ref_, &installation);

        if let Ok(mut results) = results {
            results.installed_size = bundle.installed_size();
            sender.try_send(Ok(results)).unwrap()
        } else {
            sender.try_send(results).unwrap()
        }

        Ok(())
    }

    fn run_transaction(
        &self,
        transaction_uuid: String,
        transaction: Transaction,
    ) -> Result<(), Error> {
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
        transaction_progress: &libflatpak::TransactionProgress,
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
        ref_: &Ref,
        // We need the *real* intallation (and not the dry-run clone) to check what's already
        // installed
        real_installation: &Installation,
    ) -> Result<DryRunResults, DryRunError> {
        let dry_run_results = Rc::new(RefCell::new(DryRunResults::default()));

        // Check if new remotes are added during the transaction
        transaction.connect_add_new_remote(
            clone!(@weak dry_run_results => @default-return false, move |_, reason, _, name, url|{
                if reason == TransactionRemoteReason::RuntimeDeps{
                    let remote = DryRunRemote{
                        suggested_remote_name: name.to_string(),
                        url: url.to_string(),
                    };

                    dry_run_results.borrow_mut().remotes.push(remote);
                    return true;
                }

                false
            }),
        );

        // Ready -> Everything got resolved, so we can check the transaction operations
        transaction.connect_ready(
            clone!(@weak dry_run_results, @weak ref_, @weak real_installation => @default-return false, move |transaction|{
                for operation in transaction.operations(){
                    let ref_string = ref_.format_ref().unwrap().to_string();
                    let operation_ref_string = operation.get_ref().unwrap().to_string();
                    let operation_commit = operation.commit().unwrap();

                    // Check if it's the ref which we want to install
                    if operation_ref_string == ref_string {
                        dry_run_results.borrow_mut().download_size = operation.download_size();
                        dry_run_results.borrow_mut().installed_size = operation.installed_size();

                        // Check if the ref is already installed
                        let installed = real_installation.installed_ref(
                            ref_.kind(),
                            &ref_.name().unwrap(),
                            Some(&ref_.arch().unwrap()),
                            Some(&ref_.branch().unwrap()),
                            Cancellable::NONE
                        );

                        if let Ok(installed) = installed {
                            if installed.commit().unwrap() == operation_commit {
                                // Commit is the same -> ref is already installed
                                dry_run_results.borrow_mut().is_already_done = true;
                            }else{
                                // Commit differs -> is update
                                dry_run_results.borrow_mut().is_update = true;
                            }
                        }
                    }else{
                        let runtime = DryRunRuntime{
                            ref_: operation_ref_string.to_string(),
                            type_: operation.operation_type().to_str().unwrap().to_string(),
                            download_size: operation.download_size(),
                            installed_size: operation.installed_size(),
                        };
                        dry_run_results.borrow_mut().runtimes.push(runtime);
                    }
                }

                // Do not allow the install to start, since it's a dry run
                false
            }),
        );

        if let Err(err) = transaction.run(Cancellable::NONE) {
            if err.kind::<libflatpak::Error>() == Some(libflatpak::Error::RuntimeNotFound) {
                // Unfortunately there's no clean way to find out which runtime is missing
                // so we have to parse the error message to find the runtime ref.
                let regex = regex::Regex::new(r".+ (.+/.+/[^ ]+) .+ (.+/.+/[^ ]+) .+").unwrap();
                let error = if let Some(runtimes) = regex.captures(err.message()) {
                    let runtime = runtimes[2].parse().unwrap();
                    DryRunError::RuntimeNotFound(runtime)
                } else {
                    DryRunError::RuntimeNotFound("unknown-runtime".to_string())
                };

                return Err(error);
            } else if err.kind::<libflatpak::Error>() != Some(libflatpak::Error::Aborted) {
                error!("Error during transaction dry run: {}", err.message());
                let message = err.message().to_string();
                let error = DryRunError::Other(message);
                return Err(error);
            }
        }

        let results = dry_run_results.borrow().clone();
        Ok(results)
    }

    fn new_transaction(&self, installation_uuid: &str, dry_run: bool) -> Transaction {
        let installation = self
            .installation_manager
            .flatpak_installation_by_uuid(installation_uuid);

        // Setup a own installation for dry run transactions, and add the specified
        // installation as dependency source. This way the dry run transaction
        // doesn't touch the specified installation, but has nevertheless the same local
        // runtimes available.
        if dry_run {
            let remotes = installation.list_remotes(Cancellable::NONE).unwrap();

            // Remove previous dry run installation
            std::fs::remove_dir_all("/tmp/repo").expect("Unable to remove dry run installation");

            // New temporary dry run installation
            let dry_run_path = gio::File::for_parse_name("/tmp");
            let dry_run = Installation::for_path(&dry_run_path, true, Cancellable::NONE).unwrap();

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
                dry_run
                    .add_remote(&remote_to_add, true, Cancellable::NONE)
                    .unwrap();
            }

            // Create new transaction, and add the "real" installation as dependency source
            let t = Transaction::for_installation(&dry_run, Cancellable::NONE).unwrap();
            t.add_dependency_source(&installation);

            t
        } else {
            Transaction::for_installation(&installation, Cancellable::NONE).unwrap()
        }
    }
}
