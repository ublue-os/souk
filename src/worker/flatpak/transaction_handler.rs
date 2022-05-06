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

use crate::worker::flatpak::{Command, Response};

#[derive(Debug, Clone, Downgrade)]
pub struct TransactionHandler {
    sender: Arc<Sender<Response>>,
}

impl TransactionHandler {
    pub fn start(sender: Sender<Response>, receiver: Receiver<Command>) {
        let handler = Self {
            sender: Arc::new(sender),
        };

        thread::spawn(clone!(@strong handler, @strong receiver => move || {
            let mut receiver = receiver;
            let fut = async move {
                if let Some(command) = receiver.next().await {
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
            Command::InstallFlatpakBundle(path) => self.install_flatpak_bundle(&path),
        }
        glib::Continue(true)
    }

    fn install_flatpak_bundle(&self, path: &str) {
        info!("Installing Flatpak Bundle: {}", path);
        let _file = gio::File::for_parse_name(path);

        let transaction = self.new_transaction();
        // transaction.add_install_bundle(&file, None).unwrap();
        transaction
            .add_install("gnome-nightly", "app/org.gnome.Builder/x86_64/master", &[])
            .unwrap();

        self.run_transaction(transaction);
    }

    fn run_transaction(&self, transaction: Transaction) {
        transaction.connect_new_operation(
            clone!(@weak self as this => move |transaction, operation, progress| {
                this.handle_operation(transaction, operation, progress);
            }),
        );

        transaction.connect_add_new_remote(|_transaction, _reason, _s1, _s2, _s3| {
            info!("Transaction add new remote");
            true
        });

        transaction.run(gio::Cancellable::NONE).unwrap();
    }

    fn handle_operation(
        &self,
        _transaction: &Transaction,
        operation: &TransactionOperation,
        progress: &TransactionProgress,
    ) {
        debug!(
            "Handle operation: {}",
            operation.operation_type().to_str().unwrap()
        );

        progress.connect_changed(clone!(@weak self.sender as sender => move |p|{
            let r = Response {
                progress: p.progress()
            };
            debug!("Sending response");
            sender.try_send(r).unwrap();
        }));
    }

    fn new_transaction(&self) -> Transaction {
        let installation = Installation::new_system(gio::Cancellable::NONE).unwrap();
        Transaction::for_installation(&installation, gio::Cancellable::NONE)
            .expect("Unable to create transaction")
    }
}
