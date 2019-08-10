use super::super::accounts::account; // Import the account module
use super::super::crypto::blake2; // Import the blake2 hashing module
use super::super::core::sys::{system, config}; // Import the system module

use libp2p::{PeerId, identity}; // Import the libp2p library

/// An error encountered while constructing a p2p client.
#[derive(Debug, Fail)]
pub enum ConstructionError {
    #[fail(display = "invalid p2p identity for account with address: {}", identity_hex)]
    InvalidPeerIdentity {
        identity_hex: String, // The hex encoded public key
    }
}

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
    pub fn new(network_name: &str) -> Result<Client, ConstructionError> {
        let cfg: config::Config; // Declare config buffer
        let voting_accounts: Vec<account::Account> = vec![]; // Initialize voters list
        let peer_id: PeerId; // Declare peerID buffer

        // Check peer identity exists locally
        if let Ok(p2p_account) = account::Account::read_from_disk(blake2::hash_slice(b"p2p_identity")) {
            // Check has valid p2p keypair
            if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                peer_id = PeerId::from_public_key(identity::PublicKey::Ed25519(p2p_keypair.public())); // Set peer_id in function scope
            }
        } else {
            let p2p_account = account::Account::new(); // Generate p2p account
            p2p_account.write_to_disk(); // Write p2p account to disk

            // Check has valid p2p keypair
            if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                peer_id = PeerId::from_public_key(identity::PublicKey::Ed25519(p2p_keypair.public())); // Set peer_id in function scope
            }
        }

        // Check for errors while reading config
        if let Ok(read_config) = config::Config::read_from_disk(network_name) {
            cfg = read_config; // Set config in cfg scoped buffer
        } else {
            // TODO: Download config from bootstrap nodes
        }

        Ok(Client{
            runtime: system::System::new(cfg), // Set runtime
            voting_accounts: voting_accounts, // Set voters
            peer_id: peer_id, // Set peer ID
        }) // Return initialized client
    }

    /// Initialize a new client with the given network_name and peer_id.
    pub fn with_peer_id(network_name: &str, peer_id: PeerId) -> Result<Client, ConstructionError> {
        let voting_accounts: Vec<account::Account> = vec![]; // Initialize voters list

        /// Check config exists locally
        if let Ok(read_config) = config::Config::read_from_disk(network_name) {
            Ok(with_config(read_config)) // Return initialized client
        } else {
            // TODO: Download config

            Err(ConstructionError::InvalidPeerIdentity("test")) // Return error
        }
    }

    /// Initialize a new client with the given network_name, peer_id, and config.
    pub fn with_config(network_name: &str, peer_id: PeerId, cfg: config::Config) -> Result<Client, ConstructionError> {
        let voting_accounts: Vec<account::Account> = vec![]; // Initialize voters list TODO: Add voters to list

        Ok(Client::with_voting_accounts(network_name, peer_id, cfg, voting_accounts)) // Return initialized client
    }

    // Initialize a new client with the given network_name, peer_id, config, and voting_accounts list.
    pub fn with_voting_accounts(network_name: &str, peer_id: PeerId, cfg: config::Config, voting_accounts: Vec<account::Account>) -> Client {
        Client {
            runtime: system::System::new(cfg), // Set runtime
            voting_accounts: voting_accounts, // Set voters
            peer_id: peer_id, // Set peer id
        }
    }
}
