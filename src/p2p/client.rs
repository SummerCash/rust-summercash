use super::super::accounts::account; // Import the account module
use super::super::core::sys::{config, system};
use super::super::crypto::blake2; // Import the blake2 hashing module // Import the system module

use libp2p::{identity, PeerId}; // Import the libp2p library

/// An error encountered while constructing a p2p client.
#[derive(Debug, Fail)]
pub enum ConstructionError {
    #[fail(
        display = "invalid p2p identity for account with address: {}",
        identity_hex
    )]
    InvalidPeerIdentity {
        identity_hex: String, // The hex encoded public key
    },
    #[fail(display = "an IO operation for the account {} failed", address_hex)]
    AccountIOFailed {
        address_hex: String, // The hex encoded public key
    },
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
        // Check peer identity exists locally
        if let Ok(p2p_account) =
            account::Account::read_from_disk(blake2::hash_slice(b"p2p_identity"))
        {
            // Check has valid p2p keypair
            if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                Client::with_peer_id(
                    network_name,
                    PeerId::from_public_key(identity::PublicKey::Ed25519(p2p_keypair.public())),
                ) // Return initialized client
            } else {
                Err(ConstructionError::InvalidPeerIdentity {
                    identity_hex: p2p_account.address().to_str(),
                }) // Return error
            }
        } else {
            let p2p_account = account::Account::new(); // Generate p2p account
                                                       // Write p2p account to disk
            match p2p_account.write_to_disk() {
                Ok(_) => {
                    // Check has valid p2p keypair
                    if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                        Client::with_peer_id(
                            network_name,
                            PeerId::from_public_key(identity::PublicKey::Ed25519(
                                p2p_keypair.public(),
                            )),
                        ) // Return initialized client
                    } else {
                        Err(ConstructionError::InvalidPeerIdentity {
                            identity_hex: p2p_account.address().to_str(),
                        }) // Return error
                    }
                }
                _ => Err(ConstructionError::AccountIOFailed {
                    address_hex: p2p_account.address().to_str(),
                }),
            }
        }
    }

    /// Initialize a new client with the given network_name and peer_id.
    pub fn with_peer_id(network_name: &str, peer_id: PeerId) -> Result<Client, ConstructionError> {
        // Check for errors while reading config
        if let Ok(read_config) = config::Config::read_from_disk(network_name) {
            Client::with_config(peer_id, read_config) // Return initialized client
        } else {
            // TODO: Download config

            Err(ConstructionError::InvalidPeerIdentity {
                identity_hex: "test".to_owned(),
            }) // Return error
        }
    }

    /// Initialize a new client with the given network_name, peer_id, and config.
    pub fn with_config(peer_id: PeerId, cfg: config::Config) -> Result<Client, ConstructionError> {
        let voting_accounts: Vec<account::Account> = vec![]; // Initialize voters list TODO: Add voters to list

        Ok(Client::with_voting_accounts(peer_id, cfg, voting_accounts)) // Return initialized client
    }

    // Initialize a new client with the given network_name, peer_id, config, and voting_accounts list.
    pub fn with_voting_accounts(
        peer_id: PeerId,
        cfg: config::Config,
        voting_accounts: Vec<account::Account>,
    ) -> Client {
        Client {
            runtime: system::System::new(cfg), // Set runtime
            voting_accounts: voting_accounts,  // Set voters
            peer_id: peer_id,                  // Set peer id
        }
    }
}
