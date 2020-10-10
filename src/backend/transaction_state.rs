use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionMode {
    Waiting,
    Running,
    Finished,
    Cancelled,
    Error(String),
}

impl Default for TransactionMode {
    fn default() -> Self {
        TransactionMode::Waiting
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionState {
    pub message: String,
    pub percentage: f32,
    pub mode: TransactionMode,
}

impl TransactionState {
    pub fn get_download_speed(&self) -> Option<String> {
        let message = &self.message;
        let regex = Regex::new(r"(\d+.\d+)\u{a0}(\w+)/s").unwrap();
        if let Some(speed) = regex.captures(message) {
            return Some(format!(
                "{} {}/s",
                speed[1].to_string(),
                speed[2].to_string()
            ));
        }
        None
    }
}
