use serde::{Deserialize, Serialize}; // Import serde serialization

/// A log emitted during contract execution.
#[derive(Serialize, Deserialize, Clone)]
pub struct Log<'a> {
    /// The topics of the log
    topics: Vec<String>,
    /// The message of the log
    #[serde(with = "serde_bytes")]
    message: &'a [u8],
}
