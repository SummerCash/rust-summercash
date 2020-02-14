use super::{
    core::types::{graph::Graph, transaction::Transaction},
    crypto::{blake3, hash::Hash},
};

/// A generic rule-enforcing transactional system.
pub trait Validator {
    /// Validates the contents of a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be validated
    fn transaction_is_valid(&self, tx: &Transaction) -> Result<(), failure::Error>;
}

/// A validator that is bound to the confines of a given graph. This validator presents zero cost, and requires
/// no runtime allocations.
pub struct GraphBoundValidator<'a> {
    graph: &'a Graph,
}

/// A reason provided by a GraphBoundValidator for why a particular transaction is invalid.
#[derive(Debug, Fail)]
pub enum GraphBoundValidatorReason {
    #[fail(display = "transaction {} is not unique", tx_hash)]
    NotUnique { tx_hash: Hash },
    #[fail(
        display = "transaction {} is too old; parent node {} has already been executed",
        tx_hash, invalid_parent_hash
    )]
    TooOld {
        tx_hash: Hash,
        invalid_parent_hash: Hash,
    },
    #[fail(
        display = "transaction {} is invalid; expected {}",
        tx_hash, desired_hash
    )]
    InvalidHash { tx_hash: Hash, desired_hash: Hash },
    #[fail(display = "transaction {} has an invalid signature", tx_hash)]
    InvalidSignature { tx_hash: Hash },
    #[fail(display = "transaction {} has an invalid parent receipt", tx_hash)]
    ParentReceiptInvalid { tx_hash: Hash },
}

impl<'a> GraphBoundValidator<'a> {
    /// Initializes a new validator from the provided graph.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph to which validation will be bound.
    pub fn new(graph: &'a Graph) -> Self {
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
        !self.graph.hash_routes.contains_key(&tx.hash)
    }

    /// Checks whether or not the transaction exists along an incomplete head. In other words, the transaction must be
    /// recent enough in order to be valid.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_is_head(&self, tx: &Transaction) -> (bool, Hash) {
        // Get all of the parents that the transaction relies on
        for i in 0..tx.transaction_data.parents.len() {
            // Check that the parent exists. If it doesen't, return false.
            if let Ok(Some(parent)) = self.graph.get_pure(i) {
                // The parent node shouldn't have already been resolved. The transaction is, thus, invalid.
                if parent.state_entry.is_some() {
                    return (false, parent.hash);
                }
            } else {
                // Since the transaction's parents don't exist, the tx isn't valid
                return (false, Hash::new("NILPARENT".to_owned().into_bytes()));
            }
        }

        (true, Default::default())
    }

    /// Ensure that the transaction's reported hashes are in fact valid.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked for uniqueness among the graph's txs
    fn transaction_hash_is_valid(&self, tx: &Transaction) -> (bool, Hash) {
        // Hash the transaction
        let target = blake3::hash_slice(&tx.transaction_data.to_bytes());

        // Make sure that the transaction's hash can be reproduced
        (tx.hash == target, target)
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
        if let Ok(parent_state) = self
            .graph
            .resolve_parent_nodes(tx.transaction_data.parents.clone())
        {
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

impl<'a> Validator for GraphBoundValidator<'a> {
    /// Validates the contents of a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be validated
    fn transaction_is_valid(&self, tx: &Transaction) -> Result<(), failure::Error> {
        // Ensure that all of the properties of the transaction are in fact valid
        if !self.transaction_is_unique(tx) {
            Err(GraphBoundValidatorReason::NotUnique { tx_hash: tx.hash }.into())
        } else {
            let (ok, offending_parent_hash) = self.transaction_is_head(tx);

            // Ensure that the transaction is young enough
            if !ok {
                Err(GraphBoundValidatorReason::TooOld {
                    tx_hash: tx.hash,
                    invalid_parent_hash: offending_parent_hash,
                }
                .into())
            } else if !self.transaction_signature_is_valid(tx) {
                Err(GraphBoundValidatorReason::InvalidSignature { tx_hash: tx.hash }.into())
            } else {
                // Ensure that the transaction's hash can be reproduced
                let (ok, target_hash) = self.transaction_hash_is_valid(tx);

                // If the hash can't be reproduced, the tx is invalid
                if !ok {
                    Err(GraphBoundValidatorReason::InvalidHash {
                        tx_hash: tx.hash,
                        desired_hash: target_hash,
                    }
                    .into())
                } else if !self.transaction_parent_execution_is_valid(tx) {
                    Err(GraphBoundValidatorReason::ParentReceiptInvalid { tx_hash: tx.hash }.into())
                } else {
                    Ok(())
                }
            }
        }
    }
}
