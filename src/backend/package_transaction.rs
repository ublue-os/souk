use broadcaster::BroadcastChannel;

use std::sync::Arc;
use std::sync::Mutex;

use crate::backend::{Package, PackageAction, TransactionState};

#[derive(Debug)]
pub struct PackageTransaction {
    pub package: Package,
    pub action: PackageAction,
    state: Mutex<TransactionState>,
    cancelled: Mutex<bool>,

    broadcast: BroadcastChannel<TransactionState>,
}

impl PackageTransaction {
    pub fn new(package: Package, action: PackageAction) -> Arc<Self> {
        let state = Mutex::new(TransactionState::default());
        let cancelled = Mutex::new(false);
        let broadcast = BroadcastChannel::new();

        Arc::new(Self {
            package,
            action,
            cancelled,
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

    pub fn cancel(&self){
        debug!("Cancel transaction...");
        *self.cancelled.lock().unwrap() = true;
    }

    pub fn is_cancelled(&self) -> bool{
        *self.cancelled.lock().unwrap()
    }

    fn send_message(&self, message: TransactionState) {
        let broadcast = self.broadcast.clone();
        let future = async move {
            broadcast.send(&message).await.unwrap();
        };
        spawn!(future);
    }
    //get_state()
}
