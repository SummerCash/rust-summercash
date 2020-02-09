use std::{collections, error::Error}; // Import collections

use num::bigint; // Add support for large unsigned integers

use super::super::super::crypto::hash; // Import hash types

use super::super::types::{graph, transaction}; // Import transaction types

use super::{config, proposal}; // Import parent module types

/// An error encountered while executing a proposal.
#[derive(Debug, Fail)]
pub enum ExecutionError {
    #[fail(display = "proposal with id {} does not exist", proposal_id)]
    ProposalDoesNotExist {
        proposal_id: String, // The queried key
    },
    #[fail(display = "invalid target proposal parameter {}", proposal_param)]
    InvalidTargetProposalParam {
        proposal_param: String, // The target param
    },
    #[fail(
        display = "operation {} cannot be completed on param {}",
        operation, proposal_param
    )]
    InvalidOperation {
        operation: String,      // The operation
        proposal_param: String, // The target param
    },
    #[fail(display = "{}", error)]
    Miscellaneous {
        error: String, // The error lol
    },
}

impl From<sled::Error> for ExecutionError {
    /// Converts the given sled error into an ExecutionError.
    fn from(e: sled::Error) -> Self {
        // Return a miscellaneous error
        Self::Miscellaneous {
            error: e.description().to_owned(),
        }
    }
}

/// System is a virtual proposal execution machine.
pub struct System {
    /// The system configuration
    pub config: config::Config,

    /// Known pending proposals
    pub pending_proposals: collections::HashMap<hash::Hash, proposal::Proposal>,

    /// The set fo
    pub localized_proposals: HashMap<Hash, Proposal>,

    /// The ledger
    pub ledger: graph::Graph,
}

/// Implement a set of system helper methods.
impl System {
    /// Initialize a new proposal execution system.
    pub fn new(config: config::Config) -> System {
        // Copy the network name, since we'll have to move the configuration into the system
        let network_name = &config.network_name.clone();

        System {
            config,                                                     // Set config
            pending_proposals: collections::HashMap::new(), // set pending proposals to empty initialized hash map
            ledger: graph::Graph::read_partial_from_disk(network_name), // Set ledger
        } // Return initialized system
    }

    /// Initialize a new proposal execution system with the given data directory.
    pub fn with_data_dir(config: config::Config, data_dir: &str) -> Self {
        // Copy the network name, sine we'll have to move the configuration into the system
        let network_name = &config.network_name.clone();

        System {
            config,
            pending_proposals: collections::HashMap::new(),
            ledger: graph::Graph::read_partial_from_disk_with_data_dir(data_dir, network_name),
        }
    }

    /// Add a given proposal to the system's pending proposals list.
    pub fn register_proposal(&mut self, proposal: proposal::Proposal) {
        // Check proposal not already registered
        self.pending_proposals
            .entry(proposal.proposal_id)
            .or_insert(proposal); // Add proposal
    }

    /// Execute a proposal in the pending proposals set with the given hash.
    pub fn execute_proposal(&mut self, proposal_id: hash::Hash) -> Result<(), ExecutionError> {
        // Check proposal doesn't exist
        if !self.pending_proposals.contains_key(&proposal_id) {
            Err(ExecutionError::ProposalDoesNotExist {
                proposal_id: proposal_id.to_str(),
            }) // Return error
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
                        proposal::Operation::Amend { amended_value } => {
                            self.config.reward_per_gas =
                                bigint::BigUint::from_bytes_le(&amended_value)
                        } // Set reward_per_gas
                        // Is setting reward_per_gas to zero
                        proposal::Operation::Remove => {
                            self.config.reward_per_gas = bigint::BigUint::from(0 as u16)
                        }
                        // Is adding a value to the reward_per_gas
                        proposal::Operation::Append { value_to_append } => {
                            self.config.reward_per_gas = self.config.reward_per_gas.clone()
                                + bigint::BigUint::from_bytes_le(&value_to_append)
                        } // Add to reward_per_gas
                    }

                    let operation_result = self.config.write_to_disk(); // Write config to disk
                                                                        // Check for errors
                    if let Err(e) = operation_result {
                        Err(ExecutionError::Miscellaneous {
                            error: e.to_string(),
                        })
                    } else {
                        Ok(()) // Mhm
                    }
                }
                // Proposal is targeting the network_name config field
                "config::network_name" => {
                    // Handle different operations
                    match target_proposal.proposal_data.operation {
                        // Is updating network_name
                        proposal::Operation::Amend { amended_value } => {
                            self.config.network_name =
                                String::from_utf8_lossy(&amended_value).into_owned()
                        } // Set network_name
                        // Is setting network_name to ""
                        proposal::Operation::Remove => self.config.network_name = "".to_owned(), // Set network_name to empty string
                        // Is appending a substring to the network_name
                        proposal::Operation::Append { value_to_append } => {
                            self.config.network_name = format!(
                                "{}{}",
                                self.config.network_name,
                                String::from_utf8_lossy(&value_to_append).into_owned()
                            )
                        } // Append to network_name
                    }

                    let operation_result = self.config.write_to_disk(); // Write config to disk
                    if let Err(e) = operation_result {
                        // Check for errors
                        Err(ExecutionError::Miscellaneous {
                            error: e.to_string(),
                        }) // Return error
                    } else {
                        Ok(()) // Mhm
                    }
                }
                // Proposal is targeting the ledger
                "ledger::transactions" => {
                    // Handle different operations
                    match target_proposal.proposal_data.operation {
                        // Targeted amend, despite the fact that ledger operations cannot be reverted
                        proposal::Operation::Amend { .. } => {
                            Err(ExecutionError::InvalidOperation {
                                operation: "amend".to_owned(),
                                proposal_param: "ledger::transactions".to_owned(),
                            })
                        }
                        // Targeted remove, despite the fact that ledger operations cannot be reverted
                        proposal::Operation::Remove => Err(ExecutionError::InvalidOperation {
                            operation: "remove".to_owned(),
                            proposal_param: "ledger::transactions".to_owned(),
                        }),
                        // Is appending a transaction to the network ledger
                        proposal::Operation::Append { value_to_append } => {
                            let tx = transaction::Transaction::from_bytes(&value_to_append); // Deserialize transaction

                            // Get the index of the submitted transaction entry
                            let entry_index = self.ledger.push(tx.clone(), None);

                            // Execute the parent transactions, get the overall hash
                            let parent_tx_hash =
                                self.ledger.execute_parent_nodes(entry_index)?.hash;

                            // Get the hash of the parent state that the transaction THINKS is right
                            let asserted_parent_state_hash = if let Some(parent_state_hash) =
                                tx.transaction_data.parent_state_hash
                            {
                                parent_state_hash
                            } else {
                                // Remove the head tx, since it's invalid
                                self.ledger.rollback_head();

                                // Return the error
                                return Err(ExecutionError::Miscellaneous {
                                    error: "Invalid transaction: must have parent state hash."
                                        .to_owned(),
                                });
                            };

                            // UWU WHAT'S THIS I SEE?
                            if parent_tx_hash != asserted_parent_state_hash {
                                // Remove the head tx, since it's invalid
                                self.ledger.rollback_head();

                                // Return the error
                                return Err(ExecutionError::Miscellaneous{error: "Invalid transaction: merged parent states must have a hash matching that which is asserted by the transaction.".to_owned()});
                            };

                            //if let Ok(prev_state_entry) = self
                            //    .ledger
                            //    .execute_parent_nodes(self.ledger.nodes.len() - 1)
                            //{
                            //    let index = self.ledger.nodes.len() - 1; // Get index of pushed tx

                            // Get previous state entry
                            //self.ledger.nodes[index].state_entry =
                            //Some(tx.execute(Some(prev_state_entry))); // Set node state entry
                            //}

                            let write_result = self.ledger.write_to_disk(); // Write ledger to disk
                                                                            // Check for errors
                            if let Err(e) = write_result {
                                Err(ExecutionError::Miscellaneous {
                                    error: e.to_string(),
                                }) // Return error
                            } else {
                                Ok(()) // Mhm
                            }
                        }
                    }
                }
                _ => Err(ExecutionError::InvalidTargetProposalParam {
                    proposal_param: target_proposal.proposal_data.param_name,
                }),
            }
        }
    }
}
