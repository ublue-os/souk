use broadcaster::BroadcastChannel;

use std::sync::Arc;
use std::sync::Mutex;

use crate::backend::{BasePackage, PackageAction, TransactionState};

#[derive(Debug)]
pub struct PackageTransaction {
    pub package: BasePackage,
    pub action: PackageAction,
    state: Mutex<TransactionState>,

    broadcast: BroadcastChannel<TransactionState>,
}

impl PackageTransaction {
    pub fn new(package: BasePackage, action: PackageAction) -> Arc<Self> {
        let state = Mutex::new(TransactionState::default());
        let broadcast = BroadcastChannel::new();

        Arc::new(Self {
            package,
            action,
            state,
            broadcast,
        })
    }

    pub fn get_channel(&self) -> BroadcastChannel<TransactionState> {
        self.broadcast.clone()
    }

    pub fn set_state(&self, state: TransactionState) {
        *self.state.lock().unwrap() = state.clone();
        self.send_message(state);
    }

    fn send_message(&self, message: TransactionState) {
        let broadcast = self.broadcast.clone();
        let future = async move {
            broadcast.send(&message).await.unwrap();
        };
        spawn!(future);
    }
}

impl PartialEq for PackageTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.package == other.package || self.action == other.action
    }
}
