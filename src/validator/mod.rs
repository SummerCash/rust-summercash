use super::{
    core::types::{graph::Graph, transaction::Transaction},
    crypto::blake3,
};

/// A generic rule-enforcing transactional system.
pub trait Validator {
    /// Validates the contents of a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be validated
    fn transaction_is_valid(&self, tx: &Transaction) -> bool;
}

/// A validator that is bound to the confines of a given graph. This validator presents zero cost, and requires
/// no runtime allocations.
pub struct GraphBoundValidator {
    graph: Graph,
}

impl GraphBoundValidator {
    /// Initializes a new validator from the provided graph.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph to which validation will be bound.
    pub fn new(graph: Graph) -> Self {
        // Make a new validator
        Self { graph: graph }
    }

    /// Checks whether or not the transaction already eists in the graph.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_is_unique(&self, tx: &Transaction) -> bool {
        // Return whether or not the transaction exists in the graph
        self.graph.hash_routes.contains_key(&tx.hash)
    }

    /// Checks whether or not the transaction exists along an incomplete head. In other words, the transaction must be
    /// recent enough in order to be valid.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_is_head(&self, tx: &Transaction) -> bool {
        // Get all of the parents that the transaction relies on
        for parent in tx.transaction_data.parents {
            // Check that the parent exists. If it doesen't, return false.
            if let Ok(parent) = self.graph.get_with_hash(parent) {
                // The parent node shouldn't have already been resolved. The transaction is, thus, invalid.
                if parent.state_entry.is_some() {
                    return false;
                }
            } else {
                // Since the transaction's parents don't exist, the tx isn't valid
                return false;
            }
        }

        true
    }

    /// Ensure that the transaction's reported hashes are in fact valid.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_hash_is_valid(&self, tx: &Transaction) -> bool {
        // Make sure that the transaction's hash can be reproduced
        tx.hash == blake3::hash_slice(&tx.transaction_data.to_bytes())
    }

    /// Ensures that that the signature included in the transaction is in fact valid.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_signature_is_valid(&self, tx: &Transaction) -> bool {
        // Verify the transaction's signature
        tx.verify_signature()
    }

    /// Ensures that the state transition provided by the transaction can be reproduced, and is valid.
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for a valid state transition
    fn transaction_parent_execution_is_valid(&self, tx: &Transaction) -> bool {
        // Collect the states for each of the transaction's parents
        if let Ok(parent_state) = self.graph.resolve_parent_nodes(tx.transaction_data.parents) {
            // Ensure that the transaction provides a parent state hash that we can compare the reproduced one against
            if let Some(cited_parent_hash) = tx.transaction_data.parent_state_hash {
                // Ensure that the parent hash is the same as that provided by the transaction
                parent_state.0.hash == cited_parent_hash
            } else {
                false
            }
        } else {
            // Since we can't execute the parents, this transaction must be invalid
            false
        }
    }
}

impl Validator for GraphBoundValidator {
    /// Validates the contents of a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be validated
    fn transaction_is_valid(&self, tx: &Transaction) -> bool {
        // Ensure that all of the properties of the transaction are in fact valid
        self.transaction_is_unique(tx)
            && self.transaction_is_head(tx)
            && self.transaction_hash_is_valid(tx)
            && self.transaction_signature_is_valid(tx)
            && self.transaction_parent_execution_is_valid(tx)
    }
}
