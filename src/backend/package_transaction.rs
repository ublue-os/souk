use broadcaster::BroadcastChannel;

use std::sync::Arc;
use std::sync::Mutex;

use crate::backend::{BasePackage, SoukPackageAction, SoukTransactionState};

#[derive(Debug)]
pub struct PackageTransaction {
    pub package: BasePackage,
    pub action: SoukPackageAction,
    state: Mutex<SoukTransactionState>,
    //broadcast: BroadcastChannel<SoukTransactionState>,
}

impl PackageTransaction {
    pub fn new(package: BasePackage, action: SoukPackageAction) -> Arc<Self> {
        let state = Mutex::new(SoukTransactionState::default());
        //let broadcast = BroadcastChannel::new();

        Arc::new(Self {
            package,
            action,
            state,
            //broadcast,
        })
    }

    pub fn set_state(&self, state: SoukTransactionState) {
        *self.state.lock().unwrap() = state.clone();
        self.send_message(state);
    }

    fn send_message(&self, message: SoukTransactionState) {
        //let broadcast = self.broadcast.clone();
        //let future = async move {
        //    broadcast.send(&message).await.unwrap();
        //};
        //spawn!(future);
    }
}

impl PartialEq for PackageTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.package == other.package || self.action == other.action
    }
}
