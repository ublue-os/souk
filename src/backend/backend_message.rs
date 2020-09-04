use crate::backend::PackageTransaction;

#[derive(Debug, Clone)]
pub enum BackendMessage {
    NewPackageTransaction(PackageTransaction),
    FinishedPackageTransaction(PackageTransaction),
}
