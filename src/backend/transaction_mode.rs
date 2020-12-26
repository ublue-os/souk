#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukTransactionStateKind")]
pub enum SoukTransactionMode {
    None = 0,
    Waiting = 1,
    Running = 2,
    Finished = 3,
    Cancelled = 4,
    Error = 5, // TODO: Store error message somewhere else...
}

impl Default for SoukTransactionMode {
    fn default() -> Self {
        SoukTransactionMode::Waiting
    }
}
