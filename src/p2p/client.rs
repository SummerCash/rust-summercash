use super::super::accounts::account; // Import the account module
use super::super::core::sys::{config, system}; // Import the system module
use super::super::crypto::blake2;
use super::network; // Import the network module // Import the blake2 hashing module

use std::str; // Allow libp2p to implement the write() helper method.

use log::warn; // Import logging library

use futures::future::lazy;
use libp2p::{futures::{Future, Sink}, identity, tcp::TcpConfig, websocket::WsConfig, yamux::Config, Multiaddr, PeerId, Transport, simple::SimpleProtocol}; // Import the libp2p library

use {tokio, tokio::codec::{Framed, BytesCodec}}; // Import tokio

/// An error encountered while constructing a p2p client.
#[derive(Debug, Fail)]
pub enum ConstructionError {
    #[fail(display = "invalid p2p identity")]
    InvalidPeerIdentity,
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
                Err(ConstructionError::InvalidPeerIdentity) // Return error
            }
        } else {
            let p2p_account = account::Account::new(); // Generate p2p account
                                                       // Write p2p account to disk
            match p2p_account.write_to_disk_with_name("p2p_identity") {
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
                        Err(ConstructionError::InvalidPeerIdentity) // Return error
                    }
                }
                _ => {
                    // Check could get account address
                    if let Ok(address) = p2p_account.address() {
                        Err(ConstructionError::AccountIOFailed {
                            address_hex: address.to_str(),
                        }) // Return error
                    } else {
                        Err(ConstructionError::InvalidPeerIdentity) // Return error
                    }
                }
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

            Err(ConstructionError::InvalidPeerIdentity) // Return error
        }
    }

    /// Initialize a new client with the given network_name, peer_id, and config.
    pub fn with_config(peer_id: PeerId, cfg: config::Config) -> Result<Client, ConstructionError> {
        let voting_accounts = account::get_all_unlocked_accounts(); // Get unlocked accounts

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
            voting_accounts,  // Set voters
            peer_id,                  // Set peer id
        }
    }
}

/// Broadcast a given message to a set of peers. TODO: WebSocket support, secio support
pub fn broadcast_message_raw(
    message: Vec<u8>,
    message_protocol: &str,
    network: network::Network,
    peers: Vec<Multiaddr>,
) {
    // let raw_tcp = TcpConfig::new(); // Initialize tpc config
    // let secio_upgrade = SecioConfig::new(p2p_keypair); // Initialize secio config
    // let tcp = raw_tcp.with_upgrade(secio_upgrade); // Use secio

    tokio::run(lazy(move || {
        let tcp = TcpConfig::new().or_transport(WsConfig::new(TcpConfig::new())).with_upgrade(Config::default()); // Initialize TCP/WS config
        // let tcp = raw_tcp.with_upgrade(SimpleProtocol::new(network.derive_p2p_protocol_path(message_protocol), |socket| {
        //     Ok(Framed::new(socket, BytesCodec::new())) // Return delimited
        // })); // Use network protocol

        // Iterate through peers
        for peer in peers { 
            tokio::spawn(lazy(move || {
                let msg = message.clone(); // Clone message temporarily

                if let Ok(future_conn) = tcp.clone().dial(peer.clone()) {
                    let message_send_future = future_conn.and_then(move |mut conn| conn.send(msg.as_slice().into()).map(|_| ())); // We're going to send a message, but not yet

                    let _ = tokio::run(message_send_future); // Send message
                } // Dial peer

                Ok(()) // Yup
            }));
        }

        Ok(()) // Everything's good!
    })); // Run message broadcast
}

#[cfg(test)]
mod tests {
    use num::bigint::BigUint; // Add support for large unsigned integers

    use std::str::FromStr; // Allow overriding of from_str() helper method.

    use super::*; // Import names from parent module

    #[test]
    fn test_new() {
        let config = config::Config {
            reward_per_gas: BigUint::from_str("10000000000000000000000000000000000000000").unwrap(), // Venezuela style
            network_name: "olympia".to_owned(),
        }; // Initialize config

        config.write_to_disk().unwrap(); // Write config to disk

        let client = Client::new("olympia").unwrap(); // Initialize client
        assert_eq!(client.runtime.config.network_name, "olympia"); // Ensure client has correct net
    }
}
