use crate::backend::{Package, PackageAction, TransactionState};

#[derive(Debug, Clone, PartialEq)]
pub struct PackageTransaction{
    package: Package,
    action: PackageAction,
    current_state: TransactionState,

    //bc_sender<TransactionState>,
}

impl PackageTransaction{
    pub fn new(package: Package, action: PackageAction) -> Self{
        let current_state = TransactionState::default();

        Self{
            package,
            action,
            current_state
        }
    }
    //get_state()
    //set_state()
    //get_receiver()
}
