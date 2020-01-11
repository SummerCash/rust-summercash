use super::super::accounts::account; // Import the account module
use super::super::core::sys::{config, system}; // Import the system module
use super::super::crypto::blake2; // Import the blake2 hashing module
use super::message; // Import the network module
use super::network; // Import the network module
use super::peers;
use super::sync; // Import the sync module // Import the peers module

use std::{
    collections, str,
    sync::{Arc, Mutex},
    {
        io,
        io::{Read, Write},
    },
}; // Allow libp2p to implement the write() helper method.

use libp2p::{
    futures::Future,
    identity, kad,
    tcp::{TcpConfig, TcpTransStream},
    websocket::WsConfig,
    Multiaddr, PeerId, Swarm, Transport, TransportError,
}; // Import the libp2p library

use tokio; // Import tokio

use bincode; // Import bincode

/// The global string representation for an invalid peer id.
pub static INVALID_PEER_ID_STRING: &str = "INVALID_PEER_ID";

/// An error encountered while constructing a p2p client.
#[derive(Debug, Fail)]
pub enum ConstructionError {
    #[fail(display = "invalid p2p identity")]
    InvalidPeerIdentity,
    #[fail(display = "an IO operation for the account {} failed", address_hex)]
    AccountIOFailure {
        address_hex: String, // The hex encoded public key
    },
    #[fail(display = "{}", error)]
    CommunicationsFailure { error: CommunicationError },
}

/// Implement conversion from a communication error for the ConstructionError enum.
impl From<CommunicationError> for ConstructionError {
    fn from(e: CommunicationError) -> ConstructionError {
        ConstructionError::CommunicationsFailure { error: e } // Return construction error
    }
}

/// An error encountered while communicating with another peer.
#[derive(Debug, Fail)]
pub enum CommunicationError {
    #[fail(display = "failed to serialize message")]
    MessageSerializationFailure,
    #[fail(
        display = "attempted to dial peer with address {} via an unsupported protocol",
        address
    )]
    UnsupportedProtocol { address: String },
    #[fail(display = "encountered an error while connecting to peer: {}", error)]
    IOFailure {
        error: String, // The actual error
    },
    #[fail(display = "the message was not received by a majority of specified peers")]
    MajorityDidNotReceive,
    #[fail(display = "an unknown, unexpected error occurred")]
    Unknown,
    #[fail(display = "an operation on some mutex failed")]
    MutexFailure,
    #[fail(display = "no response was received from a majority of specified peers")]
    MajorityDidNotRespond,
    #[fail(display = "no friendly peers found")]
    NoAvailablePeers,
    #[fail(display = "an error occurred while attempting a communication operation: {}", error)]
    Custom {
        error: String, // The actual error
    },
}

/// Implement conversions from a bincode error for the CommunicationError enum.
impl From<std::boxed::Box<bincode::ErrorKind>> for CommunicationError {
    fn from(_e: std::boxed::Box<bincode::ErrorKind>) -> CommunicationError {
        CommunicationError::MessageSerializationFailure // Return error
    }
}

/// Implement conversions from a libp2p transport error for the CommunicationError enum.
impl From<TransportError<std::io::Error>> for CommunicationError {
    fn from(e: TransportError<std::io::Error>) -> CommunicationError {
        // Handle different error types
        match e {
            TransportError::MultiaddrNotSupported(addr) => {
                CommunicationError::UnsupportedProtocol {
                    address: addr.to_string(),
                }
            }
            TransportError::Other(e) => CommunicationError::IOFailure {
                error: e.to_string(),
            },
        }
    }
}

/// Implement conversions from a miscellaneous error for the CommunicationError enum.
impl From<()> for CommunicationError {
    fn from(_e: ()) -> CommunicationError {
        CommunicationError::Unknown // Just return an unknown error
    }
}

/// Implement conversions from a custom error for the CommunicationError enum.
impl From<T> for CommunicationError where T: Error {
    /// Convert the error into a CommunicationError.
    fn from(e: T) -> Self {
        CommunicationError::Custom{error: e} // Return the error
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
    pub fn new(network: network::Network) -> Result<Client, ConstructionError> {
        // Check peer identity exists locally
        if let Ok(p2p_account) =
            account::Account::read_from_disk(blake2::hash_slice(b"p2p_identity"))
        {
            // Check has valid p2p keypair
            if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                Client::with_peer_id(network, identity::Keypair::Ed25519(p2p_keypair))
            // Return initialized client
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
                        Client::with_peer_id(network, identity::Keypair::Ed25519(p2p_keypair))
                    // Return initialized client
                    } else {
                        Err(ConstructionError::InvalidPeerIdentity) // Return error
                    }
                }
                _ => {
                    // Check could get account address
                    if let Ok(address) = p2p_account.address() {
                        Err(ConstructionError::AccountIOFailure {
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
    pub fn with_peer_id(
        network: network::Network,
        keypair: identity::Keypair,
    ) -> Result<Client, ConstructionError> {
        // Check for errors while reading config
        if let Ok(read_config) = config::Config::read_from_disk(network.to_str()) {
            Client::with_config(keypair, read_config) // Return initialized client
        } else {
            let config = sync::config::synchronize_for_network(
                network,
                peers::get_network_bootstrap_peer_addresses(network),
                keypair.clone()
            )?; // Get the network config file

            Client::with_config(keypair, config) // Return initialized client
        }
    }

    /// Initialize a new client with the given network_name, peer_id, and config.
    pub fn with_config(
        keypair: identity::Keypair,
        cfg: config::Config,
    ) -> Result<Client, ConstructionError> {
        let voting_accounts = account::get_all_unlocked_accounts(); // Get unlocked accounts

        Ok(Client::with_voting_accounts(keypair, cfg, voting_accounts)) // Return initialized client
    }

    // Initialize a new client with the given network_name, peer_id, config, and voting_accounts list.
    pub fn with_voting_accounts(
        keypair: identity::Keypair,
        cfg: config::Config,
        voting_accounts: Vec<account::Account>,
    ) -> Client {
        let peer_id = PeerId::from_public_key(keypair.public()); // Get peer id

        let transport = libp2p::build_tcp_ws_secio_mplex_yamux(keypair); // Build a transport

        let kad_cfg = kad::KademliaConfig::default(); // Get the default kad dht config

        let store = kad::record::store::MemoryStore::new(peer_id.clone()); // Initialize a memory store to store peer information in
        let mut behavior = kad::Kademlia::with_config(peer_id.clone(), store, kad_cfg); // Initialize a behavior from the store and kad config

        let bootstrap_addresses =
            peers::get_network_bootstrap_peers(network::Network::from(cfg.network_name.as_ref())); // Get a list of network bootstrap peers

        // Iterate through bootstrap addresses
        for bootstrap_peer in bootstrap_addresses {
            behavior.add_address(&bootstrap_peer.0, bootstrap_peer.1); // Add the bootstrap peer to the DHT
        }

        let swarm = Swarm::new(transport, behavior, peer_id.clone()); // Initialize a swarm

        Client {
            runtime: system::System::new(cfg), // Set runtime
            voting_accounts,                   // Set voters
            peer_id,                           // Set peer id
        }
    }
}

/// Broadcast a given message to a set of peers, and return the response.
pub fn broadcast_message_raw_with_response(
    message: message::Message,
    peers: Vec<Multiaddr>,
    keypair: identity::Keypair,
) -> Result<Vec<u8>, CommunicationError> {
    let mut ws_peers: Vec<Multiaddr> = vec![]; // Init WebSocket peers list
    let mut tcp_peers: Vec<Multiaddr> = vec![]; // Init TCP peers list

    // Iterate through peers
    for peer in peers.clone() {
        // Check is WebSockets peer
        if peer.to_string().contains("/ws") {
            ws_peers.push(peer); // Append peer
        } else {
            tcp_peers.push(peer); // Append peer
        }
    }

    let num_ws_peers = ws_peers.len(); // Get number of WebSockets peers
    let num_tcp_peers = tcp_peers.len(); // Get number of TCP peers

    let mut ws_resp: Result<Vec<u8>, CommunicationError> = Ok(vec![]); // Declare a buffer for ws errors
    let mut tcp_resp: Result<Vec<u8>, CommunicationError> = Ok(vec![]); // Declare a buffer for tcp errors

    // Check has any ws peers
    if ws_peers.len() > 0 {
        ws_resp = broadcast_message_raw_ws_with_response(message.clone(), ws_peers);
        // Broadcast over WS
    }
    // Check any tcp peers
    if tcp_peers.len() > 0 {
        tcp_resp = broadcast_message_raw_tcp_with_response(message, tcp_peers); // Broadcast over TCP
    }

    if num_ws_peers > num_tcp_peers {
        ws_resp // Return WebSockets response
    } else {
        tcp_resp // Return TCP error
    }
}

/// Broadcast a particular message over tcp.
fn broadcast_message_raw_tcp_with_response(
    message: message::Message,
    peers: Vec<Multiaddr>,
) -> Result<Vec<u8>, CommunicationError> {
    let serialized_message = bincode::serialize(&message)?; // Serialize message

    let tcp = TcpConfig::new(); // Initialize TCP config

    let pool = tokio::executor::thread_pool::ThreadPool::new(); // Create thread pool
    let mut dial_errors: i32 = 0; // Initialize num communication errors list

    let responses = Arc::new(Mutex::new(Vec::new())); // All responses

    let num_peers = peers.len(); // Get number of peers for later

    // Iterate through peers
    for peer in peers {
        // Dial peer
        if let Ok(conn_future) = tcp.clone().dial(peer.clone()) {
            let msg = serialized_message.clone(); // Clone message

            let mut did_connect = false; // We'll set this later if we can connect to the peer

            let responses_clone = responses.clone(); // Clone responses

            let message_send_future = conn_future
                .map_err(|e| {
                    e // Return error
                })
                .and_then(move |conn: TcpTransStream| {
                    did_connect = true; // We've successfully connected to a peer!

                    tokio::io::write_all(conn, msg) // Write to connection
                })
                .and_then(move |(mut conn, _)| {
                    let mut buf: Vec<u8> = vec![]; // Initialize empty buffer

                    // Read into response buffer
                    match conn.read_to_end(&mut buf) {
                        Err(_e) => did_connect = false,
                        _ => (),
                    };

                    // Lock responses
                    if let Ok(mut_responses) = responses_clone.lock() {
                        mut_responses.clone().push(buf); // Add to all responses
                    }

                    Ok(()) // Done!
                })
                .map_err(move |e| {
                    e // Return error
                });

            let _ = pool.spawn(lazy(move || {
                // Initialize runtime
                if let Ok(mut rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(message_send_future); // Send it!
                }

                // Check for any errors at all
                if !did_connect {
                    dial_errors += 1; // One more comm error to worry about
                }

                Ok(()) // Done!
            })); // Actually send the message
        }
    }

    pool.shutdown().wait()?; // Shutdown pool

    // Check failed to dial more than half of peers
    if dial_errors as f32 >= 0.5 * num_peers as f32 {
        Err(CommunicationError::MajorityDidNotReceive) // Return some error
    } else {
        let mut response_frequencies: collections::HashMap<Vec<u8>, i16> =
            collections::HashMap::new(); // Initialize response frequencies map

        // Do some mutex stuff first
        if let Ok(responses) = responses.lock() {
            // Get first item
            if let Some(most_common_response_ref) = responses.get(0) {
                let mut most_common_response = (*most_common_response_ref).clone(); // Clone most common response
                let mut most_common_response_frequency = 0; // Frequency of most common response
                                                            // Iterate through responses
                for response in responses.iter() {
                    let frequency = response_frequencies.entry(response.clone()).or_insert(0); // Get frequency entry
                    *frequency += 1; // Increment frequency

                    // Check is most common response
                    if *response == most_common_response {
                        most_common_response_frequency += 1; // Increment frequency
                    } else {
                        // Check has highest frequency
                        if *frequency > most_common_response_frequency {
                            most_common_response = response.clone(); // Set most common response
                            most_common_response_frequency = *frequency; // Set most common response frequency
                        }
                    }
                }

                Ok(most_common_response) // Done!
            } else {
                Err(CommunicationError::Unknown) // Idk
            }
        } else {
            Err(CommunicationError::MutexFailure) // Let the caller know we're bad at asynchronous rust
        }
    }
}

/// Broadcast a particular message over WebSockets.
fn broadcast_message_raw_ws_with_response(
    message: message::Message,
    peers: Vec<Multiaddr>,
) -> Result<Vec<u8>, CommunicationError> {
    let serialized_message = bincode::serialize(&message)?; // Serialize message

    let ws = WsConfig::new(TcpConfig::new()); // Initialize WS config

    let pool = tokio::executor::thread_pool::ThreadPool::new(); // Create thread pool
    let mut dial_errors: i32 = 0; // Initialize num communication errors list

    let responses = Arc::new(Mutex::new(Vec::new())); // All responses

    let num_peers = peers.len(); // Get number of peers for later

    // Iterate through peers
    for peer in peers {
        // Dial peer
        if let Ok(conn_future) = ws.clone().dial(peer.clone()) {
            let msg = serialized_message.clone(); // Clone message

            let mut did_connect = false; // We'll set this later if we can connect to the peer

            let responses_clone = responses.clone(); // Clone responses

            let message_send_future = conn_future
                .map_err(|_e| {
                    io::Error::from(io::ErrorKind::BrokenPipe) // Lol
                })
                .and_then(move |conn| {
                    did_connect = true; // We've successfully connected to a peer!

                    tokio::io::write_all(conn, msg) // Write to connection
                })
                .and_then(move |(mut conn, _)| {
                    let mut buf: Vec<u8> = vec![]; // Initialize empty buffer

                    // Read into response buffer
                    match conn.read_to_end(&mut buf) {
                        Err(_e) => did_connect = false,
                        _ => (),
                    };

                    // Lock responses
                    if let Ok(mut_responses) = responses_clone.lock() {
                        mut_responses.clone().push(buf); // Add to all responses
                    }

                    Ok(()) // Done!
                })
                .map_err(move |e| {
                    e // Return error
                });

            let _ = pool.spawn(lazy(move || {
                // Initialize runtime
                if let Ok(mut rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(message_send_future); // Send it!
                }

                // Check for any errors at all
                if !did_connect {
                    dial_errors += 1; // One more comm error to worry about
                }

                Ok(()) // Done!
            })); // Actually send the message
        }
    }

    pool.shutdown().wait()?; // Shutdown pool

    // Check failed to dial more than half of peers
    if dial_errors as f32 >= 0.5 * num_peers as f32 {
        Err(CommunicationError::MajorityDidNotReceive) // Return some error
    } else {
        let mut response_frequencies: collections::HashMap<Vec<u8>, i16> =
            collections::HashMap::new(); // Initialize response frequencies map

        // Do some mutex stuff first
        if let Ok(responses) = responses.lock() {
            // Get first item
            if let Some(most_common_response_ref) = responses.get(0) {
                let mut most_common_response = (*most_common_response_ref).clone(); // Clone most common response
                let mut most_common_response_frequency = 0; // Frequency of most common response
                                                            // Iterate through responses
                for response in responses.iter() {
                    let frequency = response_frequencies.entry(response.clone()).or_insert(0); // Get frequency entry
                    *frequency += 1; // Increment frequency

                    // Check is most common response
                    if *response == most_common_response {
                        most_common_response_frequency += 1; // Increment frequency
                    } else {
                        // Check has highest frequency
                        if *frequency > most_common_response_frequency {
                            most_common_response = response.clone(); // Set most common response
                            most_common_response_frequency = *frequency; // Set most common response frequency
                        }
                    }
                }

                Ok(most_common_response) // Done!
            } else {
                Err(CommunicationError::Unknown) // Idk
            }
        } else {
            Err(CommunicationError::MutexFailure) // Let the caller know we're bad at asynchronous rust
        }
    }
}

/// Broadcast a given message to a set of peers. TODO: WebSocket support, secio support, errors
pub fn broadcast_message_raw(
    message: message::Message,
    peers: Vec<Multiaddr>,
) -> Result<(), CommunicationError> {
    let mut ws_peers: Vec<Multiaddr> = vec![]; // Init WebSocket peers list
    let mut tcp_peers: Vec<Multiaddr> = vec![]; // Init TCP peers list

    // Iterate through peers
    for peer in peers.clone() {
        // Check is WebSockets peer
        if peer.to_string().contains("/ws") {
            ws_peers.push(peer); // Append peer
        } else {
            tcp_peers.push(peer); // Append peer
        }
    }

    let num_ws_peers = ws_peers.len(); // Get number of WebSockets peers
    let num_tcp_peers = tcp_peers.len(); // Get number of TCP peers

    let mut ws_err: Result<(), CommunicationError> = Ok(()); // Declare a buffer for ws errors
    let mut tcp_err: Result<(), CommunicationError> = Ok(()); // Declare a buffer for tcp errors

    // Check has any ws peers
    if ws_peers.len() > 0 {
        ws_err = broadcast_message_raw_ws(message.clone(), ws_peers); // Broadcast over WS
    }
    // Check any tcp peers
    if tcp_peers.len() > 0 {
        tcp_err = broadcast_message_raw_tcp(message, tcp_peers); // Broadcast over TCP
    }

    if num_ws_peers > num_tcp_peers {
        ws_err // Return WebSockets error
    } else {
        tcp_err // Return TCP error
    }
}

/// Broadcast a particular message over tcp.
fn broadcast_message_raw_tcp(
    message: message::Message,
    peers: Vec<Multiaddr>,
) -> Result<(), CommunicationError> {
    let serialized_message = bincode::serialize(&message)?; // Serialize message

    let tcp = TcpConfig::new(); // Initialize TCP config

    let pool = tokio::executor::thread_pool::ThreadPool::new(); // Create thread pool
    let mut dial_errors: i32 = 0; // Initialize num communication errors list

    let num_peers = peers.len(); // Get number of peers for later

    // Iterate through peers
    for peer in peers {
        // Dial peer
        if let Ok(conn_future) = tcp.clone().dial(peer.clone()) {
            let msg = serialized_message.clone(); // Clone message

            let mut did_send = true; // We'll set this later if we encounter an error sending a message
            let mut did_connect = false; // We'll set this later if we can connect to the peer

            let message_send_future = conn_future
                .map_err(|e| {
                    e // Return error
                })
                .and_then(move |mut conn| {
                    did_connect = true; // We've successfully connected to a peer!

                    conn.write_all(msg.as_slice().into()).map(|_| ()) // Write to connection
                })
                .map_err(move |e| {
                    did_send = false; // Set send error

                    e // Return error
                });

            let _ = pool.spawn(lazy(move || {
                // Initialize runtime
                if let Ok(mut rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(message_send_future); // Send it!
                }

                // Check for any errors at all
                if !did_connect || !did_send {
                    dial_errors += 1; // One more comm error to worry about
                }

                Ok(()) // Done!
            })); // Actually send the message
        }
    }

    pool.shutdown().wait()?; // Shutdown pool

    // Check failed to dial more than half of peers
    if dial_errors as f32 >= 0.5 * num_peers as f32 {
        Err(CommunicationError::MajorityDidNotReceive) // Return some error
    } else {
        Ok(()) // Done!
    }
}

/// Broadcast a particular message over WebSockets.
fn broadcast_message_raw_ws(
    message: message::Message,
    peers: Vec<Multiaddr>,
) -> Result<(), CommunicationError> {
    let serialized_message = bincode::serialize(&message)?; // Serialize message

    let ws = WsConfig::new(TcpConfig::new()); // Initialize WS config

    let pool = tokio::executor::thread_pool::ThreadPool::new(); // Create thread pool
    let mut dial_errors: i32 = 0; // Initialize num communication errors list

    let num_peers = peers.len(); // Get number of peers for later

    // Iterate through peers
    for peer in peers {
        // Dial peer
        if let Ok(conn_future) = ws.clone().dial(peer.clone()) {
            let msg = serialized_message.clone(); // Clone message

            let mut did_send = true; // We'll set this later if we encounter an error sending a message
            let mut did_connect = false; // We'll set this later if we can connect to the peer

            let message_send_future = conn_future
                .map_err(|_e| {
                    io::Error::from(io::ErrorKind::BrokenPipe) // Return error lol
                })
                .and_then(move |mut conn| {
                    did_connect = true; // We've successfully connected to a peer!

                    conn.write_all(msg.as_slice().into()).map(|_| ()) // Write to connection
                })
                .map_err(move |e| {
                    did_send = false; // Set send error

                    e // Return error
                });

            let _ = pool.spawn(lazy(move || {
                // Initialize runtime
                if let Ok(mut rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(message_send_future); // Send it!
                }

                // Check for any errors at all
                if !did_connect || !did_send {
                    dial_errors += 1; // One more comm error to worry about
                }

                Ok(()) // Done!
            })); // Actually send the message
        }
    }

    pool.shutdown().wait()?; // Shutdown pool

    // Check failed to dial more than half of peers
    if dial_errors as f32 >= 0.5 * num_peers as f32 {
        Err(CommunicationError::MajorityDidNotReceive) // Return some error
    } else {
        Ok(()) // Done!
    }
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

        let client = Client::new(network::Network::LocalTestNetwork).unwrap(); // Initialize client
        assert_eq!(client.runtime.config.network_name, "olympia"); // Ensure client has correct net
    }
}
