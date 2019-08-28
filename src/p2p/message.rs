use super::network; // Import the network module
use serde::{Deserialize, Serialize}; // Import serde serialization

/// A SummerCash network message.
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    /// Data sent in message
    pub data: Vec<u8>,
    /// The protocol associated with the message
    pub protocol: String,
    /// The networks to broadcast the message to
    pub networks: Vec<network::Network>,
}
