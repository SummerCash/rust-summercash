use super::logs; // Import the logs module

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::crypto::hash; // Import the address utility

/// A receipt of a transaction's execution.
#[derive(Serialize, Deserialize, Clone)]
pub struct Receipt<'a> {
    /// Hash of state at transaction
    pub state_hash: hash::Hash,
    /// Logs emitted at run time
    #[serde(borrow)]
    pub logs: Vec<logs::Log<'a>>,
}

/// A mapping between a set of tx hashes and transaction receipts.
#[derive(Serialize, Deserialize, Clone)]
pub struct ReceiptMap<'a> {
    /// All transactions affected by the grouped state change
    pub associated_transactions: Vec<hash::Hash>,
    /// All of the corresponding receipts
    #[serde(borrow)]
    pub state_hashes: Vec<Receipt<'a>>,
}
