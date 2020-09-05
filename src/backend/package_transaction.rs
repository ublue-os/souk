use broadcaster::BroadcastChannel;

use crate::backend::{Package, PackageAction, TransactionState};

#[derive(Debug, Clone)]
pub struct PackageTransaction {
    pub package: Package,
    pub action: PackageAction,
    state: TransactionState,

    broadcast: BroadcastChannel<TransactionState>,
}

impl PackageTransaction {
    pub fn new(package: Package, action: PackageAction) -> Self {
        let state = TransactionState::default();
        let broadcast = BroadcastChannel::new();

        Self {
            package,
            action,
            state,
            broadcast,
        }
    }

    pub fn get_channel(&self) -> BroadcastChannel<TransactionState> {
        self.broadcast.clone()
    }

    pub fn set_state(&mut self, state: TransactionState) {
        self.state = state.clone();
        self.send_message(state);
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
