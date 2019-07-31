use super::logs; // Import the logs module

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::crypto::hash; // Import the address utility

/// A receipt of a transaction's execution.
#[derive(Serialize, Deserialize, Clone)]
pub struct Receipt<'a> {
    /// Hash of state at transaction
    state_hash: hash::Hash,
    /// Logs emitted at run time
    #[serde(borrow)]
    logs: Vec<logs::Log<'a>>,
}

/// A mapping between a set of tx hashes and transaction receipts.
#[derive(Serialize, Deserialize, Clone)]
pub struct ReceiptMap<'a> {
    /// All transactions affected by the grouped state change
    associated_transactions: Vec<hash::Hash>,
    /// All of the corresponding receipts
    #[serde(borrow)]
    state_hashes: Vec<Receipt<'a>>,
}
