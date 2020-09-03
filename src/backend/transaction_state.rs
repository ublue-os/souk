#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionState {
    pub message: String,
    pub percentage: f32,
    pub is_finished: bool,
}
