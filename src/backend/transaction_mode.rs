#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukTransactionStateKind")]
pub enum SoukTransactionMode {
    Waiting = 0,
    Running = 1,
    Finished = 2,
    Cancelled = 3,
    Error = 4, // TODO: Store error message somewhere else...
}

impl Default for SoukTransactionMode {
    fn default() -> Self {
        SoukTransactionMode::Waiting
    }
}
