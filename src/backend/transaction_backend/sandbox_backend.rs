use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use regex::Regex;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{PackageAction, PackageTransaction, TransactionState};

pub struct SandboxBackend {}

impl TransactionBackend for SandboxBackend {
    fn new() -> Self {
        Self {}
    }

    fn add_package_transaction(&self, transaction: PackageTransaction) {
        debug!("New transaction: {:#?}", transaction);
        spawn!(Self::execute_package_transacton(transaction));
    }
}

impl SandboxBackend {
    async fn execute_package_transacton(mut transaction: PackageTransaction) {
        let args = Self::get_flatpak_args(&transaction);
        let mut child = Command::new("flatpak-spawn").args(&args).stdout(Stdio::piped()).spawn().unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

        while let Some(line) = lines.next().await {
            println!("{}", line.as_ref().unwrap());
            let state = Self::parse_line(line.unwrap());
            transaction.set_state(state);
        }

        debug!("Finished package transaction.");
    }

    fn get_flatpak_args(transaction: &PackageTransaction) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
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
        state.message = line.clone();

        // Regex to get percentage value
        let regex = Regex::new(r"(\d{1,3})%").unwrap();

        match regex.captures(&line) {
            Some(percentage) => {
                let value = percentage.get(1).unwrap().as_str();
                let percentage: f32 = value.parse().unwrap();
                state.percentage = percentage / 100.0;
            }
            None => (),
        }

        state
    }
}
