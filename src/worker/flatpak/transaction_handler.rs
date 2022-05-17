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
    BundleRef, Installation, Ref, Transaction, TransactionOperation, TransactionProgress,
    TransactionRemoteReason,
};

use crate::worker::flatpak;
use crate::worker::flatpak::{
    Command, DryRunError, DryRunRemote, DryRunResults, DryRunRuntime, Message, Progress,
};

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionHandler {
    transactions: Arc<Mutex<HashMap<String, Cancellable>>>,
    sender: Arc<Sender<Message>>,
}

impl TransactionHandler {
    pub fn start(sender: Sender<Message>, receiver: Receiver<Command>) {
        let handler = Self {
            transactions: Arc::default(),
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

    fn process_command(&self, command: Command) {
        debug!("Process command: {:?}", command);

        let (result, transaction_uuid) = match command {
            Command::InstallFlatpak(uuid, ref_, remote, installation) => (
                self.install_flatpak(&uuid, &ref_, &remote, &installation),
                uuid,
            ),
            Command::InstallFlatpakBundle(uuid, path, installation) => (
                self.install_flatpak_bundle(&uuid, &path, &installation),
                uuid,
            ),
            Command::InstallFlatpakBundleDryRun(path, installation, sender) => (
                self.install_flatpak_bundle_dry_run(&path, &installation, sender),
                String::new(), // We don't have an uuid for dry runs
            ),
            Command::CancelTransaction(uuid) => {
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
            // No uuid -> dry run transaction -> we don't care about errors
            if transaction_uuid.is_empty() {
                error!("Error during transaction dry run: {}", err.message());
                return;
            }

            if err.kind::<IOErrorEnum>() == Some(IOErrorEnum::Cancelled) {
                let progress = Progress::new(transaction_uuid, None, None, None);
                let progress = progress.cancelled();
                self.sender.try_send(Message::Progress(progress)).unwrap();
            } else {
                let error = flatpak::Error::new(transaction_uuid, err.message().to_string());
                self.sender.try_send(Message::Error(error)).unwrap();
            }
        }
    }

    fn install_flatpak(
        &self,
        transaction_uuid: &str,
        ref_: &str,
        remote: &str,
        installation: &str,
    ) -> Result<(), Error> {
        info!("Install Flatpak: {}", ref_);

        let transaction = self.new_transaction(installation, false);
        transaction.add_install(remote, ref_, &[])?;
        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle(
        &self,
        transaction_uuid: &str,
        path: &str,
        installation: &str,
    ) -> Result<(), Error> {
        info!("Install Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation, false);
        transaction.add_install_bundle(&file, None)?;
        self.run_transaction(transaction_uuid.to_string(), transaction)?;

        Ok(())
    }

    fn install_flatpak_bundle_dry_run(
        &self,
        path: &str,
        installation: &str,
        sender: Sender<Result<DryRunResults, DryRunError>>,
    ) -> Result<(), Error> {
        info!("Install Flatpak bundle (dry run): {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation, true);
        transaction.add_install_bundle(&file, None)?;

        let bundle = BundleRef::new(&file)?;
        let ref_ = bundle.clone().upcast::<Ref>();

        let results = self.run_dry_run_transaction(transaction, &ref_);

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
                let mut progress = Progress::new(
                    transaction_uuid.clone(),
                    Some(transaction),
                    Some(operation),
                    None,
                );

                // Check if all operations are done
                if progress.operations_count == progress.current_operation{
                    progress = progress.done();
                    this.sender.try_send(Message::Progress(progress)).unwrap();
                }else{
                    this.sender.try_send(Message::Progress(progress)).unwrap();
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

        // Transaction finished -> Remove cancellable again
        let mut transactions = self.transactions.lock().unwrap();
        transactions.remove(&transaction_uuid);

        Ok(())
    }

    fn handle_operation(
        &self,
        transaction_uuid: String,
        transaction: &Transaction,
        transaction_operation: &TransactionOperation,
        transaction_progress: &TransactionProgress,
    ) {
        let progress = Progress::new(
            transaction_uuid,
            Some(transaction),
            Some(transaction_operation),
            Some(transaction_progress),
        );
        self.sender
            .try_send(Message::Progress(progress.clone()))
            .unwrap();

        transaction_progress.connect_changed(
            clone!(@weak self.sender as sender, @strong progress => move |transaction_progress|{
                let updated = progress.update(transaction_progress);
                sender.try_send(Message::Progress(updated)).unwrap();
            }),
        );
    }

    fn run_dry_run_transaction(
        &self,
        transaction: Transaction,
        ref_: &Ref,
    ) -> Result<DryRunResults, DryRunError> {
        let download_size = Rc::new(RefCell::new(0));
        let installed_size = Rc::new(RefCell::new(0));
        let runtimes = Rc::new(RefCell::new(Vec::new()));
        let remotes = Rc::new(RefCell::new(Vec::new()));

        let cancellable = Cancellable::new();

        // Check if new remotes are added during the transaction
        transaction.connect_add_new_remote(
            clone!(@weak remotes => @default-return false, move |_, reason, _, name, url|{
                if reason == TransactionRemoteReason::RuntimeDeps{
                    let remote = DryRunRemote{
                        suggested_remote_name: name.to_string(),
                        url: url.to_string(),
                    };

                    remotes.borrow_mut().push(remote);
                    return true;
                }

                false
            }),
        );

        // Ready -> Everything got resolved, so we can check the transaction operations
        transaction.connect_ready_pre_auth(
            clone!(@weak runtimes, @weak download_size, @weak installed_size, @weak ref_ => @default-return false, move |transaction|{
                for operation in transaction.operations(){
                    let ref_ = ref_.format_ref().unwrap().to_string();
                    let operation_ref = operation.get_ref().unwrap().to_string();

                    // Check if it's the ref which we want to install
                    if operation_ref == ref_ {
                        *download_size.borrow_mut() = operation.download_size();
                        *installed_size.borrow_mut() = operation.installed_size();
                    }else{
                        let runtime = DryRunRuntime{
                            ref_: operation_ref.to_string(),
                            type_: operation.operation_type().to_str().unwrap().to_string(),
                            download_size: operation.download_size(),
                            installed_size: operation.installed_size(),
                        };
                        runtimes.borrow_mut().push(runtime);
                    }
                }

                // Do not allow the install to start, since it's a dry run
                false
            }),
        );

        if let Err(err) = transaction.run(Some(&cancellable)) {
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

        // Clean up dry run installation again
        std::fs::remove_dir_all("/tmp/repo").expect("Unable to remove dry run installation");

        let r = DryRunResults {
            download_size: 0,
            installed_size: 0,
            runtimes: runtimes.borrow().clone(),
            remotes: remotes.borrow().clone(),
        };
        Ok(r)
    }

    fn new_transaction(&self, installation: &str, dry_run: bool) -> Transaction {
        let dry_run_path = gio::File::for_parse_name("/tmp");

        let installation = match installation {
            "default" => Installation::new_system(Cancellable::NONE).unwrap(),
            "user" => {
                let mut user_path = glib::home_dir();
                user_path.push(".local/share/flatpak");
                let file = gio::File::for_path(&user_path);
                Installation::for_path(&file, true, gio::Cancellable::NONE).unwrap()
            }
            _ => panic!("Unknown Flatpak installation: {}", installation),
        };

        // Setup a own installation for dry run transactions, and add the specified
        // installation as dependency source. This way the dry run transaction
        // doesn't touch the specified installation, but has nevertheless the same local
        // runtimes available.
        if dry_run {
            let dry_run = Installation::for_path(&dry_run_path, true, Cancellable::NONE).unwrap();
            let t = Transaction::for_installation(&dry_run, gio::Cancellable::NONE).unwrap();
            t.add_dependency_source(&installation);
            t
        } else {
            Transaction::for_installation(&installation, gio::Cancellable::NONE).unwrap()
        }
    }
}
