use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use regex::Regex;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{PackageTransaction, TransactionState};

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
    async fn execute_package_transacton(transaction: PackageTransaction) {
        let mut child = Command::new("flatpak-spawn")
            .arg("--host")
            .arg("flatpak")
            .arg("install")
            .arg("--system")
            .arg("flathub")
            .arg("de.haeckerfelix.Shortwave")
            .arg("-y")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

        while let Some(line) = lines.next().await {
            let state = Self::parse_line(line.unwrap());
            debug!("Transaction state: {:?}", state);
        }
    }

    fn parse_line(line: String) -> TransactionState {
        let mut state = TransactionState::default();

        // Regex to get percentage value
        let regex = Regex::new(r"(\d{1,3})%").unwrap();

        match regex.captures(&line) {
            Some(percentage) => {
                let value = percentage.get(1).unwrap().as_str();
                state.percentage = value.parse().unwrap();
            }
            None => (),
        }

        state
    }
}
