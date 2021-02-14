use async_process::Child;
use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{
    SoukPackage, SoukPackageAction, SoukTransaction, SoukTransactionMode, SoukTransactionState,
};

type Transactions = Rc<RefCell<HashMap<String, (SoukTransaction, Child)>>>;

// Regex to get percentage value
static RE_PERCENTAGE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d{1,3})%").unwrap());

// Regex to get which package `n` out of how many packages `big_n` is being installed.
static RE_PACKAGE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)/(\d+)…").unwrap());

// Regex to parse the download speed
static RE_SPEED: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+.\d+)\u{a0}(\w+)/s").unwrap());

// Regex to checks if it is updating or installing
static RE_UPDATING: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Updating \d+/\d+…").unwrap());

pub struct SandboxBackend {
    // HashMap < app_id , ( SoukTransaction, Child ) >
    transactions: Transactions,
}

impl TransactionBackend for SandboxBackend {
    fn new() -> Self {
        let transactions = Rc::new(RefCell::new(HashMap::new()));
        Self { transactions }
    }

    fn add_transaction(&self, transaction: SoukTransaction) {
        debug!(
            "New transaction: {:?} -> {}",
            transaction.get_action(),
            transaction.get_package().get_ref_name()
        );
        spawn!(Self::execute_package_transacton(
            transaction,
            self.transactions.clone()
        ));
    }

    fn cancel_transaction(&self, transaction: SoukTransaction) {
        debug!(
            "Cancel transaction: {:?} -> {}",
            transaction.get_action(),
            transaction.get_package().get_ref_name()
        );
        let mut tupl = self
            .transactions
            .borrow_mut()
            .remove(&transaction.get_package().get_ref_name())
            .unwrap();

        match tupl.1.kill() {
            Ok(()) => {
                let state = SoukTransactionState::default();
                state.set_mode(&SoukTransactionMode::Cancelled);
                state.set_percentage(&1.0);
                transaction.set_state(&state);
                debug!("Sucessfully cancelled transaction");
            }
            Err(err) => warn!("Unable to cancel transaction: {}", err.to_string()),
        }
    }

    fn get_active_transaction(&self, package: &SoukPackage) -> Option<SoukTransaction> {
        match self.transactions.borrow().get(&package.get_ref_name()) {
            Some((t, _)) => Some(t.clone()),
            None => None,
        }
    }
}

impl SandboxBackend {
    async fn execute_package_transacton(transaction: SoukTransaction, transactions: Transactions) {
        // Set initial transaction state
        let state = SoukTransactionState::new();
        state.set_percentage(&0.0);
        transaction.set_state(&state);

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
        transactions.borrow_mut().insert(
            transaction.get_package().get_ref_name(),
            (transaction.clone(), child),
        );

        // Parse stdout lines till nothing is left anymore / the process stopped
        while let Some(line) = stdout_lines.next().await {
            let line = line.unwrap();
            debug!("Flatpak CLI: {}", line);
            let state = Self::parse_line(line);
            transaction.set_state(&state);
        }

        // Process stopped and isn't running anymore, so remove the transaction
        // from the HashMap again, and process the result / return code of it.
        match transactions
            .borrow_mut()
            .remove(&transaction.get_package().get_ref_name())
        {
            Some((_, mut child)) => {
                let state = SoukTransactionState::new();
                // Transaction finished, so let set it to 100%
                state.set_percentage(&1.0);

                // Check if it ended successfully (return code == 0)
                if child.status().await.unwrap().success() {
                    state.set_mode(&SoukTransactionMode::Finished);
                    debug!("Package transaction ended successfully.");
                } else {
                    // Get stderr information
                    let mut err_lines = String::new();
                    while let Some(line) = stderr_lines.next().await {
                        err_lines = format!("{}\n{}", err_lines, line.unwrap());
                    }

                    // TODO: Transfer error message somewhere else
                    //state.mode = SoukTransactionMode::Error(err_lines);
                    state.set_mode(&SoukTransactionMode::Error);
                    debug!("Package transaction did not end successfully.");
                }

                // Set last transaction state.
                transaction.set_state(&state);
            }
            // When we cancel the transaction before, it isn't available in the HashMap anymore.
            None => debug!("Unable to end package transaction. Probably got cancelled before."),
        };
    }

    fn get_flatpak_args(transaction: &SoukTransaction) -> Vec<String> {
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
        match transaction.get_action() {
            SoukPackageAction::Install => {
                args.push("install".into());
                args.push("--system".into());
                args.push(transaction.get_package().get_remote().clone());
                args.push(transaction.get_package().get_name().clone());
                args.push("-y".into());
            }
            SoukPackageAction::Uninstall => {
                args.push("uninstall".into());
                args.push("--system".into());
                args.push(transaction.get_package().get_name().clone());
                args.push("-y".into());
            }
            _ => (),
        };

        args
    }

    fn parse_line(line: String) -> SoukTransactionState {
        let state = SoukTransactionState::new();
        state.set_mode(&SoukTransactionMode::Running);

        let mut n: f64 = 1.0;
        let mut big_n: f64 = 1.0;

        if let Some(percentage) = RE_PERCENTAGE.captures(&line) {
            let value = percentage.get(1).unwrap().as_str();
            let percentage = value.parse::<f64>().unwrap() / 100.0;

            if let Some(package_number) = RE_PACKAGE_NUMBER.captures(&line) {
                n = package_number[1].parse().unwrap();
                big_n = package_number[2].parse().unwrap();
                let global_percentage = (n - 1.0 + percentage) / big_n;
                state.set_percentage(&global_percentage);
            } else {
                state.set_percentage(&percentage);
            }
        }

        let mut message = String::new();

        // In case the number of packages is 1 his condistion just checks,
        // if the percentage is lower than 0.99.
        if state.get_percentage() < n / big_n - 0.01 {
            if let Some(speed) = RE_SPEED.captures(&line) {
                message = format!(
                    "Downloading {} {}/s",
                    speed[1].to_string(),
                    speed[2].to_string()
                );
            } else {
                let re = regex::Regex::new(r"^Looking for matches…$").unwrap();
                if re.is_match(&line) {
                    message = "Preparing…".to_string();
                }
            }
        } else {
            if RE_UPDATING.is_match(&line) {
                message = "Updating…".to_string();
            } else {
                message = "Installing…".to_string();
            }
        }
        state.set_message(&message);

        state
    }
}
