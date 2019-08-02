use super::logs; // Import the logs module

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::crypto::hash; // Import the address utility

/// A receipt of a transaction's execution.
#[derive(Serialize, Deserialize, Clone)]
pub struct Receipt {
    /// Hash of state at transaction
    pub state_hash: hash::Hash,
    /// Logs emitted at run time
    pub logs: Vec<logs::Log>,
}

/// A mapping between a set of tx hashes and transaction receipts.
#[derive(Serialize, Deserialize, Clone)]
pub struct ReceiptMap {
    /// All transactions affected by the grouped state change
    pub associated_transactions: Vec<hash::Hash>,
    /// All of the corresponding receipts
    pub receipts: Vec<Receipt>,
}
