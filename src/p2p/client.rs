use super::super::accounts::account; // Import the account module
use super::super::core::sys::{system, config}; // Import the system module

use libp2p::PeerId; // Import the libp2p library

/// A network client.
pub struct Client {
    /// The active SummerCash runtime environment
    pub runtime: system::System,
    /// The list of accounts used to vote on proposals
    pub voting_accounts: Vec<account::Account>,
    /// The client's libp2p peer identity
    pub peer_id: PeerId,
}

/// Implement a set of client helper methods.
impl Client {
    pub fn new(network_name: &str) -> Client {
        let cfg: config::Config; // Declare config buffer
        let voting_accounts: Vec<account::Account> = vec![]; // Initialize voters list
        let peer_id: PeerId; // Declare peerID buffer

        // Check for errors while reading config
        if let Ok(read_config) = config::Config::read_from_disk(network_name) {
            cfg = read_config; // Set config in cfg scoped buffer
        } else {
            // TODO: Download config from bootstrap nodes
        }

        Client{
            runtime: system::System::new(cfg), // Set runtime
            voting_accounts: voting_accounts, // Set voters
            peer_id: peer_id, // Set peer ID
        } // Return initialized client
    }
}
