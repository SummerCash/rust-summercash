use super::logs; // Import the logs module

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::crypto::hash::Hash; // Import the address utility

/// A receipt of a transaction's execution.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Receipt {
    /// Hash of state at transaction
    pub state_hash: Hash,
    /// Logs emitted at run time
    pub logs: Vec<logs::Log>,
}

/// A mapping between a set of tx hashes and transaction receipts.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ReceiptMap {
    /// All transactions affected by the grouped state change
    pub associated_transactions: Vec<Hash>,
    /// All of the corresponding receipts
    pub receipts: Vec<Receipt>,
}

impl ReceiptMap {
    /// Gets a receipt stored in the receiptmap by its hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the receipt that should be retrieved
    pub fn receipt_for_transaction(&self, hash: Hash) -> Option<&Receipt> {
        // Look for a matching receipt
        for i in 0..self.associated_transactions.len() {
            if self.associated_transactions[i] == hash {
                // We've found a matching receipt; return it
                return self.receipts.get(i);
            }
        }

        None
    }

    /// Determines whether or not a state entry is existent in the ReceiptMap with the provided hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the entry that should exist in the map
    pub fn contains_receipt_with_state_hash(&self, hash: Hash) -> bool {
        // Look for a receipt with a matching state hash
        for i in 0..self.receipts.len() {
            if self.receipts[i].state_hash == hash {
                return true;
            }
        }

        false
    }
}
