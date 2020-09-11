use std::sync::Arc;

use crate::backend::PackageTransaction;

#[derive(Debug, Clone)]
pub enum BackendMessage {
    PackageTransaction(Arc<PackageTransaction>),
}
