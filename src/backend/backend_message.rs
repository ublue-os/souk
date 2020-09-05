use crate::backend::PackageTransaction;

#[derive(Debug, Clone)]
pub enum BackendMessage {
    PackageTransaction(PackageTransaction),
}
