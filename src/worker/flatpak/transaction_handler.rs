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

use std::sync::Arc;
use std::thread;

use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use async_std::task;
use glib::{clone, Downgrade, Error};
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{Installation, Transaction, TransactionOperation, TransactionProgress};

use crate::worker::flatpak;
use crate::worker::flatpak::{Command, Message, Progress};

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionHandler {
    sender: Arc<Sender<Message>>,
}

impl TransactionHandler {
    pub fn start(sender: Sender<Message>, receiver: Receiver<Command>) {
        let handler = Self {
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

    fn process_command(&self, command: Command) -> glib::Continue {
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
        };

        if let Err(err) = result {
            let error = flatpak::Error::new(transaction_uuid, err.message().to_string());
            self.sender.try_send(Message::Error(error)).unwrap();
        }

        glib::Continue(true)
    }

    fn install_flatpak(
        &self,
        transaction_uuid: &str,
        ref_: &str,
        remote: &str,
        installation: &str,
    ) -> Result<(), Error> {
        info!("Installing Flatpak: {}", ref_);

        let transaction = self.new_transaction(installation);
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
        info!("Installing Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation);
        transaction.add_install_bundle(&file, None)?;
        self.run_transaction(transaction_uuid.to_string(), transaction)?;

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
                    transaction,
                    operation,
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

        // TODO: This might send doubled error messages
        transaction.connect_operation_error(
            clone!(@weak self as this, @strong transaction_uuid => @default-return false,  move |_transaction, _operation, error, _details| {
                let error = flatpak::Error::new(transaction_uuid.clone(), error.message().to_string());
                this.sender.try_send(Message::Error(error)).unwrap();

                false
            }),
        );

        transaction.run(gio::Cancellable::NONE)
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
            transaction,
            transaction_operation,
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

    fn new_transaction(&self, installation: &str) -> Transaction {
        let installation = match installation {
            "default" => Installation::new_system(gio::Cancellable::NONE).unwrap(),
            _ => panic!("Unknown Flatpak installation: {}", installation),
        };

        Transaction::for_installation(&installation, gio::Cancellable::NONE)
            .expect("Unable to create transaction")
    }
}
