mod host_backend;

mod sandbox_backend;
pub use sandbox_backend::SandboxBackend;

use crate::backend::{SoukPackage, SoukTransaction};

pub trait TransactionBackend {
    fn new() -> Self
    where
        Self: Sized;

    fn add_transaction(&self, transaction: SoukTransaction);

    fn cancel_package_transaction(&self, transaction: SoukTransaction);

    fn get_active_transaction(&self, package: &SoukPackage) -> Option<SoukTransaction>;
}
