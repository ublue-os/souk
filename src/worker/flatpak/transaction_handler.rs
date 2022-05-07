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
use glib::{clone, Downgrade};
use gtk::{gio, glib};
use libflatpak::prelude::*;
use libflatpak::{Installation, Transaction, TransactionOperation, TransactionProgress};

use crate::worker::flatpak::{Command, Progress};

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionHandler {
    sender: Arc<Sender<Progress>>,
}

impl TransactionHandler {
    pub fn start(sender: Sender<Progress>, receiver: Receiver<Command>) {
        let handler = Self {
            sender: Arc::new(sender),
        };

        thread::spawn(clone!(@strong handler, @strong receiver => move || {
            let mut receiver = receiver;
            let fut = async move {
                while let Some(command) = receiver.next().await {
                    task::spawn_blocking(clone!(@weak handler => move || {
                        handler.process_command(command);
                    })).await;
                }
            };
            task::block_on(fut);
        }));
    }

    fn process_command(&self, command: Command) -> glib::Continue {
        debug!("Process command: {:?}", command);

        match command {
            Command::InstallFlatpakBundle(path, installation) => {
                self.install_flatpak_bundle(&path, &installation)
            }
        }
        glib::Continue(true)
    }

    fn install_flatpak_bundle(&self, path: &str, installation: &str) {
        info!("Installing Flatpak bundle: {}", path);
        let file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction(installation);
        transaction.add_install_bundle(&file, None).unwrap();

        self.run_transaction(transaction);
    }

    fn run_transaction(&self, transaction: Transaction) {
        transaction.connect_new_operation(
            clone!(@weak self as this => move |transaction, operation, progress| {
                this.handle_operation(transaction, operation, progress);
            }),
        );

        transaction.run(gio::Cancellable::NONE).unwrap();
    }

    fn handle_operation(
        &self,
        transaction: &Transaction,
        transaction_operation: &TransactionOperation,
        transaction_progress: &TransactionProgress,
    ) {
        let progress = Progress::new(transaction, transaction_operation, transaction_progress);
        self.sender.try_send(progress.clone()).unwrap();

        transaction_progress.connect_changed(
            clone!(@weak self.sender as sender, @strong progress => move |transaction_progress|{
                let updated = progress.update(transaction_progress);
                sender.try_send(updated).unwrap();
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
