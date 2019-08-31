use super::super::super::crypto::{blake2, hash}; // Import the blake2 hashing module

use serde::{Deserialize, Serialize}; // Import serde serialization

/// A proposal regarding a network-wide action.
#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    /// The name of the proposal
    pub proposal_name: String,
    /// The body of the proposal
    pub proposal_data: ProposalData,
    /// The hash of the proposal
    pub proposal_id: hash::Hash,
}

/// The body of a proposal.
#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalData {
    /// The name of the system parameter to modify
    pub param_name: String,
    /// The manner in which to modify the parameter (e.g. "amend")
    pub operation: Operation,
}

/// The manner in which a particular atomic event should be treated.
#[derive(Serialize, Deserialize, Clone)]
pub enum Operation {
    /// Make a minor change, or revision to a particular attribute or event
    Amend { amended_value: Vec<u8> },
    /// Remove a particular attribute or event from the network's shared memory
    Remove,
    /// Add a value to a particular attribute or set of events
    Append { value_to_append: Vec<u8> },
}

/// A vector of proposals.
#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalList {
    /// The proposals
    pub proposals: Vec<Proposal>,
}

/// Implement a set of proposal helper methods.
impl Proposal {
    /// Initialize a new Proposal instance with the given parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::core::sys::proposal; // Import proposal types
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    /// use std::str::FromStr; // Allow overriding of from_str() helper method
    ///
    /// let operation = proposal::Operation::Amend {
    ///     amended_value: BigUint::from_str("10").unwrap().to_bytes_le(), // Set amended value
    /// }; // Initialize operation
    ///
    /// let proposal = proposal::Proposal::new("test_proposal".to_owned(), proposal::ProposalData::new("reward_per_gas".to_owned(), operation)); // Initialize proposal
    /// ```
    pub fn new(proposal_name: String, proposal_data: ProposalData) -> Proposal {
        let mut proposal = Proposal {
            proposal_name: proposal_name, // Set proposal name
            proposal_data: proposal_data, // Set proposal data
            proposal_id: hash::Hash::new(vec![0; hash::HASH_SIZE]), // Set id to empty hash
        }; // Initialize proposal

        proposal.proposal_id = blake2::hash_slice(
            serde_json::to_vec_pretty(&proposal.proposal_data.clone())
                .unwrap()
                .as_slice(),
        ); // Set proposal id

        proposal // Return proposal
    }

    /// Encode &self to a byte vector via serde_json.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap() // Return serialized
    }
}

/// Implement a set of proposal data helper methods.
impl ProposalData {
    /// Initialize a new ProposalData instance with the given parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::core::sys::proposal; // Import proposal types
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    /// use std::str::FromStr; // Allow overriding of from_str() helper method
    ///
    /// let operation = proposal::Operation::Amend {
    ///     amended_value: BigUint::from_str("10").unwrap().to_bytes_le(), // Set amended value
    /// }; // Initialize operation
    ///
    /// let proposal_data = proposal::ProposalData::new("reward_per_gas".to_owned(), operation); // Initialize proposal data
    /// ```
    pub fn new(param_name: String, operation: Operation) -> ProposalData {
        ProposalData {
            param_name: param_name, // Set param name
            operation: operation,   // Set operation
        } // Return initialized proposal data
    }
}

/// Implement a serialization for the proposal list type.
impl ProposalList {
    /// Encode the list of proposals to a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap() // Return serialized
    }
}
