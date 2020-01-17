use super::super::accounts::account; // Import the account module
use super::super::core::sys::{
    config::{self, Config},
    system,
}; // Import the system module
use super::super::crypto::blake3; // Import the blake3 hashing module
use super::network; // Import the network module
use super::peers;
use std::{
    error::Error,
    io, str,
    sync::{Arc, Mutex},
}; // Allow libp2p to implement the write() helper method.

use super::sync::{self, context::Ctx};

use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    identity, kad,
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent, Record},
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmEvent},
    Multiaddr, NetworkBehaviour, PeerId, Swarm, TransportError,
}; // Import the libp2p library

// We need these traits from the futures library in order to build a swarm.
use futures::io::{AsyncRead, AsyncWrite};

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

/// Implement conversion from an IO error for the ConstructionError enum.
impl From<io::Error> for ConstructionError {
    /// Converts the given IO error into a ConstructionError.
    fn from(e: io::Error) -> Self {
        // Return the error
        Self::CommunicationsFailure { error: e.into() }
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
    #[fail(
        display = "an error occurred while attempting a communication operation: {}",
        error
    )]
    Custom {
        error: String, // The actual error
    },
}

// Implement conversions from a libp2p transport error for the CommunicationError enum.
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

impl From<io::Error> for CommunicationError {
    /// Converts the given IO error to a CommunicationError.
    fn from(e: io::Error) -> Self {
        // Return the IO error as a CommunicationError
        Self::IOFailure {
            error: e.description().to_owned(),
        }
    }
}

impl From<sled::Error> for CommunicationError {
    /// Converts the given IO error to a CommunicationError.
    fn from(e: sled::Error) -> Self {
        // Return an IO error to represent the sled error, since that's basically that DB interaction is
        Self::IOFailure {
            error: e.description().to_owned(),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for CommunicationError {
    /// Converts the given mutex poison error to a CommunicationError.
    fn from(e: std::sync::PoisonError<T>) -> Self {
        // Just return a misc. error, since we aren't really sure why this happened
        Self::Custom {
            error: e.description().to_owned(),
        }
    }
}

/// A network behavior describing a client connected to a pub-sub compatible,
/// optionally mDNS-compatible network. Such a "behavior" may be implemented for
/// any libp2p transport, but any transport used with this behavior must implement
/// asynchronous reading & writing capabilities.
#[derive(NetworkBehaviour)]
pub struct ClientBehavior<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> {
    /// Some pubsub mechanism bound to the above transport
    pub floodsub: Floodsub<TSubstream>,

    /// Some mDNS service bound to the above transport
    pub mdns: Mdns<TSubstream>,

    /// Allow for the client to do some external discovery on the global network through a KAD DHT
    pub kad_dht: Kademlia<TSubstream, MemoryStore>,
}

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> ClientBehavior<TSubstream> {
    /// Adds the given peer with a particular ID & multi address to the behavior.
    pub fn add_address(&mut self, id: &PeerId, multi_address: Multiaddr) {
        // Add the peer to the KAD DHT
        self.kad_dht.add_address(id, multi_address);

        // Add the peer to the list of floodsub peers to message
        self.floodsub.add_node_to_partial_view(id.clone());
    }
}

/*
    BEGIN IMPLEMENTATION OF DISCOVERY VIA mDNS & KAD EVENTS
*/

/// Discovery via mDNS events.
impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<MdnsEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming mDNS message from a potential peer. Add them to the local registry if the connection succeeds.
    fn inject_event(&mut self, event: MdnsEvent) {
        // Check what kind of packet the peer has sent us, and, from there, decide what we want to do with it.
        match event {
            MdnsEvent::Discovered(list) =>
            // Go through each of the peers we were able to connect to, and add them to the localized node registry
            {
                for (peer, _) in list {
                    // Log the discovered peer to stdout
                    debug!("Received mDNS 'alive' confirmation from peer: {}", peer);

                    // Register the discovered peer in the localized pubsub service instance
                    self.floodsub.add_node_to_partial_view(peer)
                }
            }
            MdnsEvent::Expired(list) =>
            // Go through each of the peers we were able to connect to, and remove them from the localized node registry
            {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        // Log the peer that will be removed
                        info!("Peer {} dead; removing", peer);

                        // Oops, rent is up, and the bourgeoisie haven't given up their power. I guess it's time to die, poor person. Sad proletariat.
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

/// Network synchronization via KAD DHT events.
/// Synchronization of network proposals, for example, is done in this manner.
/// TODO: Not implemented
impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<KademliaEvent> for ClientBehavior<TSubstream>
{
    // Wait for a peer to send us a kademlia event message. Once this happens, we can try to use the message for something (e.g. synchronization).
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            // The record was found successfully; print it
            KademliaEvent::GetRecordResult(Ok(result)) => {
                for Record { key, value: _, .. } in result.records {
                    // Print out the record
                    info!("Found key: {:?}", key);
                }
            }
            // An error occurred while fetching the record; print it
            KademliaEvent::GetRecordResult(Err(e)) => warn!("Failed to load record: {:?}", e),
            _ => {}
        }
    }
}

/*
    END IMPLEMENTATION OF DISCOVERY VIA mDNS & KAD EVENTS
*/

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<FloodsubEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming floodsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, _message: FloodsubEvent) {
        // TODO: Unimplemented
    }
}

/// A network client.
pub struct Client {
    /// The active SummerCash runtime environment
    pub runtime: system::System,

    /// The list of accounts used to vote on proposals
    pub voting_accounts: Vec<account::Account>,

    /// The client's libp2p peer identity keypair
    pub keypair: identity::Keypair,

    /// The client's libp2p peer identity
    pub peer_id: PeerId,
}

impl Into<String> for &Client {
    /// Converts the given client into a string.
    fn into(self) -> String {
        // The collected accounts in a siingle string
        let mut accounts_string = String::new();

        // Iterate through the accounts in the client configuration
        for i in 0..self.voting_accounts.len() {
            if let Ok(addr) = self.voting_accounts[i].address() {
                accounts_string += hex::encode(addr).as_ref();
            }
        }

        format!(
            "primary voting account: {},\npeer ID: {}",
            accounts_string,
            self.peer_id.to_base58(),
        )
    }
}

/// Implement a set of client helper methods.
impl Client {
    pub fn new(network: network::Network, data_dir: &str) -> Result<Client, ConstructionError> {
        // Check peer identity exists locally
        if let Ok(p2p_account) = account::Account::read_from_disk_at_data_directory(
            blake3::hash_slice(b"p2p_identity"),
            data_dir,
        ) {
            // Check has valid p2p keypair
            if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                Client::with_peer_id(network, identity::Keypair::Ed25519(p2p_keypair), data_dir)
            // Return initialized client
            } else {
                Err(ConstructionError::InvalidPeerIdentity) // Return error
            }
        } else {
            let p2p_account = account::Account::new(); // Generate p2p account
                                                       // Write p2p account to disk
            match p2p_account.write_to_disk_with_name_at_data_directory("p2p_identity", data_dir) {
                Ok(_) => {
                    // Check has valid p2p keypair
                    if let Ok(p2p_keypair) = p2p_account.p2p_keypair() {
                        Client::with_peer_id(
                            network,
                            identity::Keypair::Ed25519(p2p_keypair),
                            data_dir,
                        )
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
        data_dir: &str,
    ) -> Result<Client, ConstructionError> {
        // Check for errors while reading config
        if let Ok(read_config) = config::Config::read_from_disk(network.into()) {
            Ok(Client::with_config(keypair, read_config, data_dir)) // Return initialized client
        } else {
            let config = Config {
                reward_per_gas: config::DEFAULT_REWARD_PER_GAS.into(),
                network_name: network.into(),
            };

            Ok(Client::with_config(keypair, config, data_dir)) // Return initialized client
        }
    }

    /// Initialize a new client with the given network_name, peer_id, and config.
    pub fn with_config(keypair: identity::Keypair, cfg: config::Config, data_dir: &str) -> Client {
        let voting_accounts = account::get_all_unlocked_accounts(); // Get unlocked accounts

        // Initialize a client with the given keypair, configuration & voting accounts
        Client::with_voting_accounts(keypair, cfg, voting_accounts, data_dir)
    }

    // Initialize a new client with the given network_name, peer_id, config, and voting_accounts list.
    pub fn with_voting_accounts(
        keypair: identity::Keypair,
        cfg: config::Config,
        voting_accounts: Vec<account::Account>,
        data_dir: &str,
    ) -> Client {
        // Return the initialized client inside a result
        Client {
            runtime: system::System::with_data_dir(cfg, data_dir), // Set runtime
            voting_accounts,                                       // Set voters
            peer_id: PeerId::from_public_key(keypair.public()),    // Set peer id
            keypair,
        }
    }

    /// Starts the client.
    pub async fn start(&self) -> io::Result<()> {
        let store = kad::record::store::MemoryStore::new(self.peer_id.clone()); // Initialize a memory store to store peer information in

        // Initialize a new behavior for a client that we will generate in the not-so-distant future with the given peerId, alongside
        // an mDNS service handler as well as a floodsub instance targeted at the given peer
        let mut behavior = ClientBehavior {
            floodsub: Floodsub::new(self.peer_id.clone()),
            mdns: Mdns::new().await?,
            kad_dht: Kademlia::new(self.peer_id.clone(), store),
        };

        let bootstrap_addresses = peers::get_network_bootstrap_peers(network::Network::from(
            self.runtime.config.network_name.as_ref(),
        )); // Get a list of network bootstrap peers

        // Log the pending bootstrap operation
        info!("Bootstrapping a network DHT & behavior to existing bootstrap nodes...");

        // The current bp #
        let mut i: usize = 0;

        // Iterate through bootstrap addresses
        for bootstrap_peer in bootstrap_addresses {
            // Log the pending connection op
            info!(
                "Connecting to bootstrap node {} ({})...",
                i, bootstrap_peer.1
            );

            behavior.add_address(&bootstrap_peer.0, bootstrap_peer.1); // Add the bootstrap peer to the DHT

            // Next bp...
            i += 1;
        }

        // Start bootstrapping the DHT to the peers we've connected to
        info!("Bootstrapping the network DHT to the connected peers");

        // Bootstrap the behavior's DHT
        behavior.kad_dht.bootstrap();

        let mut swarm = Swarm::new(
            libp2p::build_tcp_ws_secio_mplex_yamux(self.keypair.clone())?,
            behavior,
            self.peer_id.clone(),
        ); // Initialize a swarm

        // Try to get the address we'll listen on
        if let Ok(addr) = "/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>() {
            // Try to tell the swarm to listen on this address, return an error if this doesn't work
            if let Err(e) = Swarm::listen_on(&mut swarm, addr.clone()) {
                // Log the error
                error!("Swarm failed to bind to listening address {}: {}", addr, e);

                // Return an error
                return Err(io::ErrorKind::AddrNotAvailable.into());
            };
            // Print the address we'll be listening on
            info!("Swarm listening on addr {}; ready for connections", addr);

            // Make a context buffer that the node can use to synchronize with
            let ctx: Mutex<Ctx> = Mutex::new(Ctx::new());

            // Get a thread-safe reference to the context for the swarm thread
            let swarm_ctx = Arc::new(ctx);

            // Get a thread-safe reference to the context for the handler thread
            let handler_ctx = swarm_ctx.clone();
            let handler_dht = Arc::new(Mutex::new(swarm.kad_dht));
            let handler_runtime = Arc::new(Mutex::new(self.runtime));

            // Start a handler thread
            std::thread::spawn(move || Client::handler(handler_ctx, handler_dht, handler_runtime));

            loop {
                // Poll the swarm
                match swarm.next_event().await {
                    // Info from the swarm is really all we care about
                    SwarmEvent::Behaviour(e) => {
                        info!("idk: {:?}", e);
                    }

                    // Just do some logging with the excess data
                    SwarmEvent::Connected(peer_id) => {
                        debug!("Connected to peer: {}", peer_id.to_base58())
                    }
                    SwarmEvent::Disconnected(peer_id) => {
                        debug!("Disconnected from peer: {}", peer_id.to_base58())
                    }
                    SwarmEvent::NewListenAddr(l_addr) => {
                        info!("Assigned to new address; listening on {} now", l_addr)
                    }
                    SwarmEvent::ExpiredListenAddr(e_addr) => {
                        info!("Listener address {} expired", e_addr)
                    }
                    SwarmEvent::UnreachableAddr {
                        peer_id: _,
                        address,
                        error,
                    } => warn!("Failed to connect to peer at addr {}: {}", address, error),
                    SwarmEvent::StartConnect(peer_id) => debug!(
                        "Starting connection process with peer: {}",
                        peer_id.to_base58()
                    ),
                }
            }
        } else {
            // Log the error
            error!("Swarm failed to bind to listening address");

            // Return an error that says we can't listen on this address
            return Err(io::ErrorKind::AddrNotAvailable.into());
        }
    }

    /// Handles any incoming changes from the swarm, and starts the synchronization sequence.
    fn handler<TSubstream>(
        ctx_lock: Arc<Mutex<Ctx>>,
        dht_lock: Arc<Mutex<Kademlia<TSubstream, MemoryStore>>>,
        runtime_lock: Arc<Mutex<system::System>>,
    ) -> Result<(), failure::Error> {
        // Try to get a lock on the context buffer. If this fails, something has gone HORRIBLY wrong.
        let mut ctx = if let Ok(unlocked) = ctx_lock.lock() {
            unlocked
        } else {
            return Ok(());
        };

        // Try to get a lock on the runtime.
        let mut runtime = if let Ok(unlocked) = runtime_lock.lock() {
            unlocked
        } else {
            return Ok(());
        };

        // Try to get a lock on the DHT.
        let mut dht = if let Ok(unlocked) = dht_lock.lock() {
            unlocked
        } else {
            return Ok(());
        };

        // If there aren't any transactions in the DAG, let's start with the root
        let current_transaction_hash = if runtime.ledger.nodes.len() == 0 {
            // Get the root node in the entire transaction history
            let root_tx = sync::synchronize_root_transaction_for_network(&mut ctx, &mut dht)?;

            // Keep a copy of the node's hash, since we'll dispose of the rest of it once it's back in the graph
            let node_hash = root_tx.hash.clone();

            // Add the node to the graph
            runtime.ledger.add(root_tx);

            // Return the hash of the node as the current transaction hash
            node_hash
        } else {
            // Use the last transaction, since one exists
            runtime.ledger.nodes[runtime.ledger.nodes.len() - 1].hash
        };

        // All good!
        Ok(())
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

        let client = Client::new(
            network::Network::LocalTestNetwork,
            super::super::super::common::io::DATA_DIR,
        )
        .unwrap(); // Initialize client
        assert_eq!(client.runtime.config.network_name, "olympia"); // Ensure client has correct net
    }
}
