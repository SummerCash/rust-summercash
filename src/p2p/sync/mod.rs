use libp2p::kad::{
    record::{store::MemoryStore, Key},
    Kademlia, Quorum,
}; // Import the libp2p library

use std::collections::HashMap;

use super::{
    super::{
        core::{
            sys::{config::Config, proposal::Proposal},
            types::graph::{Graph, Node},
        },
        crypto::hash::Hash,
    },
    client::CommunicationError,
};

use context::Ctx;

pub mod context;

/// Every time we want to synchronize the local DAG, we'll download & then purge 10 transactions at a time.
pub const TRANSACTIONS_PER_SYNCHRONIZATION_ROUND: u8 = 10;

/// The byte-representation fo the proposals key
pub const PROPOSALS_KEY: &[u8] = b"proposals";

/// Represents the DHT Key for the network configuration.
pub const CONFIG_KEY: &[u8] = b"config";

/// Represents the DHT Key for the root entry in the DAG.
pub const ROOT_TRANSACTION_KEY: &[u8] = b"ledger::transactions::root";

/// Represents the DHT Key for the head (latest) entry in the DAG.
pub const HEAD_TRANSACTION_KEY: &[u8] = b"ledger::transactions::head";

/// Represenst the DHT Key for the next entry in the DAG.
pub const NEXT_TRANSACTION_KEY: &[u8] = b"ledger::transactions::next";

/// Represents the DHT Key for an entry in the DAG with a particular hash.
pub const TRANSACTION_KEY: &[u8] = b"ledger::transactions::tx";

/// Constructs a new NEXT_TRANSACTION_KEY from the given hash.
pub fn next_transaction_key(hash: Hash) -> Key {
    // Format the normal next tx path with the given hash
    Key::new(&format!("ledger::transactions::next({})", hash.to_str()).as_bytes())
}

/// Constructs a new TRANSACTION_KEY from the given hash.
pub fn transaction_with_hash_key(hash: Hash) -> Key {
    // Format the normal transaction path with the given hash
    Key::new(&format!("ledger::transactions::tx({})", hash.to_str()).as_bytes())
}

/// Download a copy of the network's list of proposals.
///
/// # Arguments
///
/// * `dht` - An instance of the Kademlia DHT
/// * `network` - The network
pub fn synchronize_proposals_for_network<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
) -> Result<HashMap<Hash, Proposal>, CommunicationError> {
    dht.get_record(&Key::new(&PROPOSALS_KEY), Quorum::Majority);

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
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
) -> Result<Config, CommunicationError> {
    // Perform a query on the KAD DHT for the network's configuration
    dht.get_record(&Key::new(&CONFIG_KEY), Quorum::Majority);

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

/// Synchronize a local transaction graph against a remote copy.
pub fn synchronize_dag_for_network_against_head<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
    dag: &mut Graph,
) -> Result<(), CommunicationError> {
    // If the graph doesn't even have a root transaction, return an error
    if dag.nodes.len() == 0 {
        // Return an appropriate error
        return Err(CommunicationError::Custom {
            error: "No root node exists in the provided DAG.".to_owned(),
        });
    }

    // Reset the context
    ctx.flush();

    // Sync the target transaction hash
    let target_tx_hash = synchronize_head_transaction_hash_for_network(ctx, dht)?;

    // The current transaction hash
    let mut current_tx_hash = dag.nodes[0].hash;

    // The number of transactions synchronized in the current batch
    let mut n_transactions_synchronized = 0;

    // Wait until we're ahead of the target transaction
    while current_tx_hash != target_tx_hash {
        // Get the hash of the next transaction in the chain
        current_tx_hash = synchronize_next_transaction_hash_for_network(ctx, dht, current_tx_hash)?;

        // Get the actual node corresponding to the current hash
        let current_node =
            synchronize_transaction_with_hash_for_network(ctx, dht, current_tx_hash)?;

        // Put the node in the local graph instance
        dag.add(current_node);

        // Reset the # of synchronized transactions if we've completed the batch
        if n_transactions_synchronized >= TRANSACTIONS_PER_SYNCHRONIZATION_ROUND {
            // Save the disk to the drive & purge
            dag.write_to_disk()?;

            // Remove all of the old batch stuff from the DAG--if we need it later, we can reload from the DB.
            dag.purge();

            n_transactions_synchronized = 0;
        }

        // Increment the number of transactions that we've synchronized
        n_transactions_synchronized += 1;
    }

    // We're done!
    Ok(())
}

/// Synchronize a copy of the network's root transaction node.
pub fn synchronize_root_transaction_for_network<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
) -> Result<Node, CommunicationError> {
    // Perform a query on the KAD DHT for the network's configuration
    dht.get_record(&Key::new(&ROOT_TRANSACTION_KEY), Quorum::Majority);

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

/// Synchronize a copy of the next transaction from the network.
pub fn synchronize_next_transaction_hash_for_network<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
    current_node_hash: Hash,
) -> Result<Hash, CommunicationError> {
    // Perform a query
    dht.get_record(
        &Key::new(&next_transaction_key(current_node_hash)),
        Quorum::Majority,
    );

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

/// Synchronize a copy of the head transaction against the remote network.
pub fn synchronize_head_transaction_hash_for_network<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
) -> Result<Hash, CommunicationError> {
    // Perform a query on the KAD DHT for the network's head transaction hash
    dht.get_record(&Key::new(&HEAD_TRANSACTION_KEY), Quorum::Majority);

    // Try to get a reesponse for the query. If nothing is returned,
    // err.
    if let Some(resp) = ctx.response() {
        // Return the response
        Ok(resp)
    } else {
        // Return an error
        Err(CommunicationError::MajorityDidNotRespond)
    }
}

/// Synchronize a copy of a node with the given hash rom the remote network.
pub fn synchronize_transaction_with_hash_for_network<TSubstream>(
    ctx: &mut Ctx,
    dht: &mut Kademlia<TSubstream, MemoryStore>,
    hash: Hash,
) -> Result<Node, CommunicationError> {
    // Perform a query on the KAD DHT for a transaction with the corresponding hash
    dht.get_record(
        &Key::new(&transaction_with_hash_key(hash)),
        Quorum::Majority,
    );

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
