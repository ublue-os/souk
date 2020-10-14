use async_process::Child;
use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use regex::Regex;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{
    BasePackage, Package, PackageAction, PackageTransaction, TransactionMode, TransactionState,
};

type Transactions = Rc<RefCell<HashMap<String, (Arc<PackageTransaction>, Child)>>>;

pub struct SandboxBackend {
    // HashMap < app_id , ( PackageTransaction, Child ) >
    transactions: Transactions,
}

impl TransactionBackend for SandboxBackend {
    fn new() -> Self {
        let transactions = Rc::new(RefCell::new(HashMap::new()));
        Self { transactions }
    }

    fn add_package_transaction(&self, transaction: Arc<PackageTransaction>) {
        debug!(
            "New transaction: {:?} -> {}",
            transaction.action,
            transaction.package.ref_name()
        );
        spawn!(Self::execute_package_transacton(
            transaction,
            self.transactions.clone()
        ));
    }

    fn cancel_package_transaction(&self, transaction: Arc<PackageTransaction>) {
        debug!(
            "Cancel transaction: {:?} -> {}",
            transaction.action,
            transaction.package.ref_name()
        );
        let mut tupl = self
            .transactions
            .borrow_mut()
            .remove(&transaction.package.ref_name())
            .unwrap();

        match tupl.1.kill() {
            Ok(()) => {
                let mut state = TransactionState::default();
                state.mode = TransactionMode::Cancelled;
                state.percentage = 1.0;
                transaction.set_state(state);
                debug!("Sucessfully cancelled transaction");
            }
            Err(err) => warn!("Unable to cancel transaction: {}", err.to_string()),
        }
    }

    fn get_active_transaction(
        &self,
        package: &BasePackage,
    ) -> Option<std::sync::Arc<PackageTransaction>> {
        match self.transactions.borrow().get(&package.ref_name()) {
            Some((t, _)) => Some(t.clone()),
            None => None,
        }
    }
}

impl SandboxBackend {
    async fn execute_package_transacton(
        transaction: Arc<PackageTransaction>,
        transactions: Transactions,
    ) {
        // Set initial transaction state
        let mut state = TransactionState::default();
        state.percentage = 0.0;
        transaction.set_state(state);

        // Setup flatpak child / procress and spawn it
        let args = Self::get_flatpak_args(&transaction);
        let mut child = Command::new("flatpak-spawn")
            .args(&args)
            .env("LANG", "C")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        // We're going to parse the lines to get status information
        let mut stdout_lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut stderr_lines = BufReader::new(child.stderr.take().unwrap()).lines();

        // Insert running child into transaction HashMap, we need to access it later...
        // 1) when we want to cancel the transaction
        // 2) when we want to know the current state of a running transaction
        transactions
            .borrow_mut()
            .insert(transaction.package.ref_name(), (transaction.clone(), child));

        // Parse stdout lines till nothing is left anymore / the process stopped
        while let Some(line) = stdout_lines.next().await {
            let state = Self::parse_line(line.unwrap());
            transaction.set_state(state);
        }

        // Process stopped and isn't running anymore, so remove the transaction
        // from the HashMap again, and process the result / return code of it.
        match transactions
            .borrow_mut()
            .remove(&transaction.package.ref_name())
        {
            Some((_, mut child)) => {
                let mut state = TransactionState::default();
                // Transaction finished, so let set it to 100%
                state.percentage = 1.0;

                // Check if it ended successfully (return code == 0)
                if child.status().await.unwrap().success() {
                    state.mode = TransactionMode::Finished;
                    debug!("Package transaction ended successfully.");
                } else {
                    // Get stderr information
                    let mut err_lines = String::new();
                    while let Some(line) = stderr_lines.next().await {
                        err_lines = format!("{}\n{}", err_lines, line.unwrap());
                    }

                    state.mode = TransactionMode::Error(err_lines);
                    debug!("Package transaction did not end successfully.");
                }

                // Set last transaction state.
                transaction.set_state(state);
            }
            // When we cancel the transaction before, it isn't available in the HashMap anymore.
            None => debug!("Unable to end package transaction. Probably got cancelled before."),
        };
    }

    fn get_flatpak_args(transaction: &PackageTransaction) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        // If we kill flatpak-spawn, we also want to kill the child process too.
        args.push("--watch-bus".into());
        // We cannot do stuff inside the Flatpak Sandbox,
        // so we have to spawn it on the host side.
        args.push("--host".into());
        args.push("flatpak".into());

        // Generate the Flatpak command
        // Note: The command cannot ask any further questions,
        // everything must run automatically, so we set "-y" everywhere.
        match transaction.action {
            PackageAction::Install => {
                args.push("install".into());
                args.push("--system".into());
                args.push(transaction.package.remote().clone());
                args.push(transaction.package.name().clone());
                args.push("-y".into());
            }
            PackageAction::Uninstall => {
                args.push("uninstall".into());
                args.push("--system".into());
                args.push(transaction.package.name().clone());
                args.push("-y".into());
            }
            _ => (),
        };

        args
    }

    fn parse_line(line: String) -> TransactionState {
        let mut state = TransactionState::default();
        state.mode = TransactionMode::Running;

        // Regex to get percentage value
        let regex = Regex::new(r"(\d{1,3})%").unwrap();

        let mut n: f32 = 1.0;
        let mut big_n: f32 = 1.0;
        if let Some(percentage) = regex.captures(&line) {
            let value = percentage.get(1).unwrap().as_str();
            let percentage = value.parse::<f32>().unwrap() / 100.0;

            // Regex to get which package `n` out of how many packages `big_n`
            // is being installed.
            let re_package_number = Regex::new(r"(\d+)/(\d+)…").unwrap();
            if let Some(package_number) = re_package_number.captures(&line) {
                n = package_number[1].parse().unwrap();
                big_n = package_number[2].parse().unwrap();
                let global_percentage = (n - 1.0 + percentage) / big_n;
                state.percentage = global_percentage;
            } else {
                state.percentage = percentage;
            }
        }

        // When the number of packages is 1, this just means, if percentage
        // is lower than 0.99.
        if state.percentage < n / big_n - 0.01 {
            let re = Regex::new(r"(\d+.\d+)\u{a0}(\w+)/s").unwrap();
            if let Some(speed) = re.captures(&line) {
                state.message = format!(
                    "Downloading {} {}/s",
                    speed[1].to_string(),
                    speed[2].to_string()
                );
            } else {
                let re = Regex::new(r"^Looking for matches…$").unwrap();
                if re.is_match(&line) {
                    state.message = "Preparing…".to_string();
                }
            }
        } else {
            let re = Regex::new(r"^Updating \d+/\d+…").unwrap();
            if re.is_match(&line) {
                state.message = "Updating…".to_string();
            } else {
                state.message = "Installing…".to_string();
            }
        }

        state
    }
}
