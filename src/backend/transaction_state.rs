#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionState{
    message: String,
    percentage: f32,
    is_finished: bool,
}
