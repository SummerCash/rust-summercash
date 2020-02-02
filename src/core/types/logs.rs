use serde::{Deserialize, Serialize}; // Import serde serialization

/// A log emitted during contract execution.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Log {
    /// The topics of the log
    pub topics: Vec<String>,
    /// The message of the log
    pub message: Vec<u8>,
}
