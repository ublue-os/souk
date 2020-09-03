use crate::backend::PackageTransaction;

#[derive(Debug, Clone, PartialEq)]
pub enum BackendMessage {
    NewPackageTransaction(PackageTransaction),
    FinishedPackageTransaction(PackageTransaction),
}
