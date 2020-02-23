use super::{
    common::address::Address,
    core::types::{graph::Graph, transaction::Transaction},
    crypto::{blake3, hash::Hash},
};
use num::BigUint;

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
    #[fail(
        display = "the balance of the sender of transaction {} ({}) is insufficient to execute such a state transition",
        tx_hash, sender
    )]
    InsufficientSenderBalance { tx_hash: Hash, sender: Address },
}

impl<'a> GraphBoundValidator<'a> {
    /// Initializes a new validator from the provided graph.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph to which validation will be bound.
    pub fn new(graph: &'a Graph) -> Self {
        // Make a new validator
        Self { graph }
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
            // The hash of the parental node
            let parent = tx.transaction_data.parents[i];

            // If we know what the index of this parent transaction is in the state graph, we can
            // try to pull out a fully-formed state matching this parent transaction
            if let Some(parent_index) = self.graph.hash_routes.get(&parent) {
                // Check that the parent exists. If it doesen't, return false.
                if let Ok(Some(parent)) = self.graph.get_pure(*parent_index) {
                    // The parent node shouldn't have already been resolved. The transaction is, thus, invalid.
                    if parent.state_entry.is_some() {
                        return (false, parent.hash);
                    }
                } else {
                    // Since the transaction's parents don't exist, the tx isn't valid
                    return (false, Hash::new("NILPARENT".to_owned().into_bytes()));
                }
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
        let target =
            blake3::hash_slice(&bincode::serialize(&tx.transaction_data).unwrap_or_default());

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
    ///
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

    /// Ensures that the sender of the provided transaction has enough SummerCash to perform the transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction that should be checked against
    fn transaction_sender_balance_is_sufficient(&self, tx: &Transaction) -> bool {
        // Check for a latest state entry in the graph. This will serve as the point from where we calculate the account's balance.
        if let Some(last_state) = self.graph.obtain_executed_head() {
            // Ensure that the provided transaction has in fact been executed
            if let Some(state) = last_state.state_entry {
                // The sender must have at least enough coins to send the transaction
                return *state
                    .data
                    .balances
                    .get(&tx.transaction_data.sender.to_str())
                    .unwrap_or(&BigUint::default())
                    >= tx.transaction_data.value;
            }
        }

        // If the sender doesn't have any SMC, they can't send any. Therefore, the value of the transaction must be zero.
        tx.transaction_data.value == BigUint::default()
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
                    // If the user sending the transaction doesn't have enough SMC to actually send this transaction, return an error
                    if !self.transaction_sender_balance_is_sufficient(tx) {
                        Err(GraphBoundValidatorReason::InsufficientSenderBalance {
                            tx_hash: tx.hash,
                            sender: tx.transaction_data.sender,
                        }
                        .into())
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }
}
