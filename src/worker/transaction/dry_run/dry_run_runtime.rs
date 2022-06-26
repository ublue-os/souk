use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Default, Deserialize, Serialize, Type, Debug, Clone)]
pub struct DryRunRuntime {
    pub ref_: String,
    pub operation_type: String,
    pub download_size: u64,
    pub installed_size: u64,
}
