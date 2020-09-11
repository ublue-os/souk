use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use regex::Regex;
use async_process::Child;

use std::sync::Arc;
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{Package, PackageAction, PackageTransaction, TransactionMode, TransactionState};

type Transactions = Rc<RefCell<HashMap<String, (Arc<PackageTransaction>, Child)>>>;

pub struct SandboxBackend {
    transactions: Transactions,
}

impl TransactionBackend for SandboxBackend {
    fn new() -> Self {
        let transactions = Rc::new(RefCell::new(HashMap::new()));
        Self {transactions}
    }

    fn add_package_transaction(&self, transaction: Arc<PackageTransaction>) {
        debug!("New transaction: {:?} -> {}", transaction.action, transaction.package.get_ref_name());
        spawn!(Self::execute_package_transacton(transaction, self.transactions.clone()));
    }

    fn cancel_package_transaction(&self, transaction: Arc<PackageTransaction>){
        debug!("Calcel transaction: {:?} -> {}", transaction.action, transaction.package.get_ref_name());
        let mut tupl = self.transactions.borrow_mut().remove(&transaction.package.get_ref_name()).unwrap();

        match tupl.1.kill(){
            Ok(()) => {
                let mut state = TransactionState::default();
                state.mode = TransactionMode::Cancelled;
                state.percentage = 1.0;
                transaction.set_state(state);
                debug!("Sucessfully cancelled transaction");
            }
            Err(err) => warn!("Unable to cancel transaction: {}", err.to_string())
        }
    }
}

impl SandboxBackend {
    async fn execute_package_transacton(transaction: Arc<PackageTransaction>, transactions: Transactions) {
        // Set initial transaction state
        let mut state = TransactionState::default();
        state.percentage = 0.0;
        transaction.set_state(state);

        let args = Self::get_flatpak_args(&transaction);
        let mut child = Command::new("flatpak-spawn")
            .args(&args)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        transactions.borrow_mut().insert(transaction.package.get_ref_name(), (transaction.clone(), child));

        while let Some(line) = lines.next().await {
            println!("{}", line.as_ref().unwrap());
            let state = Self::parse_line(line.unwrap());
            transaction.set_state(state);
        }

        match transactions.borrow_mut().remove(&transaction.package.get_ref_name()){
            Some(_) => {
                // Finish transaction
                let mut state = TransactionState::default();
                state.percentage = 1.0;
                state.mode = TransactionMode::Finished;
                transaction.set_state(state);
                debug!("Package transaction ended successfully.");
            },
            None => debug!("Unable to end package transaction. Probably got cancelled before."),
        };
    }

    fn get_flatpak_args(transaction: &PackageTransaction) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        args.push("--watch-bus".into());
        args.push("--host".into());
        args.push("flatpak".into());

        match transaction.action {
            PackageAction::Install => {
                args.push("install".into());
                args.push("--system".into());
                args.push(transaction.package.remote.clone());
                args.push(transaction.package.app_id.clone());
                args.push("-y".into());
            }
            PackageAction::Uninstall => {
                args.push("uninstall".into());
                args.push("--system".into());
                args.push(transaction.package.app_id.clone());
                args.push("-y".into());
            }
            _ => (),
        };

        args
    }

    fn parse_line(line: String) -> TransactionState {
        let mut state = TransactionState::default();
        state.mode = TransactionMode::Running;
        state.message = line.clone();

        // Regex to get percentage value
        let regex = Regex::new(r"(\d{1,3})%").unwrap();

        if let Some(percentage) = regex.captures(&line) {
            let value = percentage.get(1).unwrap().as_str();
            let percentage: f32 = value.parse().unwrap();
            state.percentage = percentage / 100.0;
        }

        state
    }
}
