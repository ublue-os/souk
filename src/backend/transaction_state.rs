#[derive(Debug, Clone, PartialEq)]
pub enum TransactionMode{
    Waiting,
    Running,
    Finished,
    Cancelled,
    Error(String),
}

impl Default for TransactionMode {
    fn default() -> Self { TransactionMode::Waiting }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionState {
    pub message: String,
    pub percentage: f32,
    pub mode: TransactionMode,
}
