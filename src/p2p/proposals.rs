use super::{network, client, super::{crypto::hash, core::sys::proposal}}; // Import the network, hash, proposal modules

use std::collections::HashMap; // Import the hashmap type

/// Download a copy of the network's list of proposals.
pub fn synchronize_for_network(network: network::Network) -> Result<HashMap<hash::Hash, proposal::Proposal>, client::CommunicationError> {
    
}