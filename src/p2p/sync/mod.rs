pub mod context;

use std::{collections::HashMap, sync::Arc};

use super::{
    super::{
        core::{
            sys::{config::Config, proposal::Proposal},
            types::graph::Graph,
        },
        crypto::hash::Hash,
    },
    client::CommunicationError,
};

use context::Ctx;

use libp2p::kad::{
    record::{store::MemoryStore, Key},
    Kademlia, Quorum,
}; // Import the libp2p library

/// Download a copy of the network's list of proposals.
///
/// # Arguments
///
/// * `dht` - An instance of the Kademlia DHT
/// * `network` - The network
pub fn synchronize_proposals_for_network<TSubstream>(
    ctx: Arc<Ctx<HashMap<Hash, Proposal>>>,
    dht: Kademlia<TSubstream, MemoryStore>,
) -> Result<HashMap<Hash, Proposal>, CommunicationError> {
    dht.get_record(&Key::new(b"proposals"), Quorum::Majority);

    // Try to get a response for the query. If nothing is returned,
    // err.
    if let Some(resp) = ctx.response() {
        // Return the response
        Ok(resp)
    } else {
        // Return an error
        Err(CommunicationError::MajorityDidNotRespond)
    }
}

/// Compare a local copy of the configuration file to a remote
///
/// # Arguments
///
/// * `dht` - An instance of the Kademlia DHT
/// * `existing_config` - The network's configuration file
/// * `network` - The name of the desired network to connect to
/// * `keypair` - The keypair to encrypt connections with
pub fn synchronize_configuration_for_network<TSubstream>(
    ctx: Arc<Ctx<Config>>,
    dht: Kademlia<TSubstream, MemoryStore>,
) -> Result<Config, CommunicationError> {
    // Perform a query on the KAD DHT for the network's configuration
    dht.get_record(&Key::new(b"config"), Quorum::Majority);
}

/// Synchronize a local transaction graph against a remote copy.
pub fn synchronize_dag_for_network_against_head<TSubstream>(
    dht: Kademlia<TSubstream, MemoryStore>,
    dag: &mut Graph,
) -> Result<(), CommunicationError> {
    // Ensure that the dag has a head we can work off.
    // If the graph doesn't have a head transaction, we'll have to synchronize it from the network.
    // We can use the quorum for this. Thus, this synchronization query will be retried once the
    // head transaction has cleared trough the quorum.
    if dag.nodes.len() == 0 {}
}

/// Synchronize a copy of the network's root transaction node.
pub fn synchronize_root_transaction_for_network<TSubstream>(
    dht: Kademlia<TSubstream, MemoryStore>,
) {
    // Perform a query on the KAD DHT for the network's configuration
    dht.get_record(&Key::new(b"ledger::transactions::root"), Quorum::Majority);
}

/// Synchronize a copy of the next transaction from the network.
pub fn synchronize_next_transaction_hash_for_network<TSubstream>(
    dht: Kademlia<TSubstream, MemoryStore>,
    current_node_hash: Hash,
) {
    // Perform a query
    dht.get_record(
        &Key::new::<&[u8]>(&format!("ledger::transactions::next({})", current_node_hash).as_ref()),
        Quorum::Majority,
    );
}

/// Synchronize a copy of the head transaction against the remote network.
pub fn synchronize_head_transaction_for_network<TSubstream>(
    dht: Kademlia<TSubstream, MemoryStore>,
) {
    // Perform a query on the KAD DHT for the network's head transaction hash
    dht.get_record(&Key::new(b"ledger::transactions::head"), Quorum::Majority);
}
