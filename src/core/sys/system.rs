use std::collections; // Import collections

use num::bigint; // Add support for large unsigned integers

use super::super::super::crypto::hash; // Import hash types

use super::super::types::{transaction, graph}; // Import transaction types

use super::{proposal, config}; // Import parent module types

/// An error encountered while executing a proposal
#[derive(Debug, Fail)]
pub enum ExecutionError {
    #[fail(
        display = "proposal with id {} does not exist",
        proposal_id
    )]
    ProposalDoesNotExist {
        proposal_id: String,   // The queried key
    },
    #[fail(
        display = "invalid target proposal parameter {}",
        proposal_param
    )]
    InvalidTargetProposalParam {
        proposal_param: String, // The target param
    },
    #[fail(
        display = "operation {} cannot be completed on param {}",
        operation,
        proposal_param,
    )]
    InvalidOperation {
        operation: String, // The operation
        proposal_param: String, // The target param
    },
    #[fail(
        display = "{}",
        error
    )]
    Miscellaneous {
        error: String, // The error lol
    }
}

/// System is a virtual proposal execution machine.
pub struct System {
    /// The system configuration
    pub config: config::Config,
    /// Known pending proposals
    pub pending_proposals: collections::HashMap<hash::Hash, proposal::Proposal>,
    /// The ledger
    pub ledger: graph::Graph,
}

/// Implement a set of system helper methods.
impl System {
    /// Initialize a new proposal execution system.
    pub fn new(config: config::Config) -> System {
        System{
            config: config, // Set config
            pending_proposals: collections::HashMap::new(), // set pending proposals to empty initialized hash map
            ledger: graph::Graph::read_partial_from_disk(&config.network_name), // Set ledger
        } // Return initialized system
    }

    /// Execute a proposal in the pending proposals set with the given hash.
    pub fn execute_proposal(mut self, proposal_id: hash::Hash) -> Result<(), ExecutionError> {
        // Check proposal doesn't exist
        if !self.pending_proposals.contains_key(&proposal_id) {
            Err(ExecutionError::ProposalDoesNotExist{proposal_id: proposal_id.to_str()}) // Return error
        } else {
            let target_proposal = self.pending_proposals.get(&proposal_id).unwrap().clone(); // Get proposal

            self.pending_proposals.remove(&proposal_id); // Remove proposal from pending proposals

            // Handle different target system parameters
            match target_proposal.proposal_data.param_name.as_str() {
                // Proposal is targeting the reward_per_gas config field
                "config::reward_per_gas" => {
                    // Handle different operations
                    match target_proposal.proposal_data.operation {
                        // Is updating reward_per_gas
                        proposal::Operation::Amend{amended_value} => self.config.reward_per_gas = bigint::BigUint::from_bytes_le(&amended_value), // Set reward_per_gas
                        // Is setting reward_per_gas to zero
                        proposal::Operation::Remove => self.config.reward_per_gas = bigint::BigUint::from(0 as u16),
                        // Is adding a value to the reward_per_gas
                        proposal::Operation::Append{value_to_append} => self.config.reward_per_gas = self.config.reward_per_gas + bigint::BigUint::from_bytes_le(&value_to_append), // Add to reward_per_gas
                    }

                    let operation_result = self.config.write_to_disk(); // Write config to disk
                    if operation_result.is_err() { // Check for errors
                        Err(ExecutionError::Miscellaneous{error: operation_result.unwrap_err().to_string()})
                    } else {
                        Ok(()) // Mhm
                    }
                },
                // Proposal is targeting the network_name config field
                "config::network_name" => {
                    // Handle different operations
                    match target_proposal.proposal_data.operation {
                        // Is updating network_name
                        proposal::Operation::Amend{amended_value} => self.config.network_name = String::from_utf8_lossy(&amended_value).into_owned(), // Set network_name
                        // Is setting network_name to ""
                        proposal::Operation::Remove => self.config.network_name = "".to_owned(), // Set network_name to empty string
                        // Is appending a substring to the network_name
                        proposal::Operation::Append{value_to_append} => self.config.network_name = format!("{}{}", self.config.network_name, String::from_utf8_lossy(&value_to_append).into_owned()), // Append to network_name
                    }

                    let operation_result = self.config.write_to_disk(); // Write config to disk
                    if operation_result.is_err() { // Check for errors
                        Err(ExecutionError::Miscellaneous{error: operation_result.unwrap_err().to_string()}) // Return error
                    } else {
                        Ok(()) // Mhm
                    }
                },
                // Proposal is targeting the ledger
                "ledger::transactions" => {
                    // Handle different operations
                    match target_proposal.proposal_data.operation {
                        // Targeted amend, despite the fact that ledger operations cannot be reverted
                        proposal::Operation::Amend{amended_value} => Err(ExecutionError::InvalidOperation{operation: "amend".to_owned(), proposal_param: "ledger::transactions".to_owned()}),
                        // Targeted remove, despite the fact that ledger operations cannot be reverted
                        proposal::Operation::Remove => Err(ExecutionError::InvalidOperation{operation: "remove".to_owned(), proposal_param: "ledger::transactions".to_owned()}),
                        // Is appending a transaction to the network ledger
                        proposal::Operation::Append{value_to_append} => {
                            let tx = transaction::Transaction::from_bytes(&value_to_append); // Deserialize transaction

                            self.ledger.push(tx, None); // Add tx to ledger

                            if tx.transaction_data.parent_state_hash.is_some() { // Check parent state hash cleared
                                for parent in tx.transaction_data.parents { // Iterate through parents
                                    let parent_node_result = self.ledger.get_with_hash(parent); // Get parent node
                                    if parent_node_result.is_err() { // Check for errors
                                        Err(ExecutionError::Miscellaneous{error: parent_node_result.unwrap_err().to_string()}) // Return error
                                    } else {
                                        let parent_nodes: Vec<graph::Node>; // Init parent nodes vec
                                        for parent_of_parent in parent_node_result.unwrap().transaction.
                                        self.ledger.update(self.ledger.hash_routes.get(&parent).unwrap(), parent_node_result.unwrap().execute())

                                        Ok(()) // Mhm
                                    }
                                }
                            }

                            let write_result = self.ledger.write_to_disk(); // Write ledger to disk
                            if write_result.is_err() { // Check for errors
                                Err(ExecutionError::Miscellaneous{error: write_result.unwrap_err().to_string()}) // Return error
                            } else {
                                Ok(()) // Mhm
                            }
                        },
                    }
                },
                _ => Err(ExecutionError::InvalidTargetProposalParam{proposal_param: target_proposal.proposal_data.param_name})
            }
        }
    }
}