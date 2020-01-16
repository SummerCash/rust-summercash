pub mod context;

use std::{collections::HashMap, sync::Arc};

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

use libp2p::kad::{
    record::{store::MemoryStore, Key},
    Kademlia, Quorum,
}; // Import the libp2p library

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
pub const TRANSACTION_KEY: &[u8] = b"ledger::transactions::transaction";

/// Constructs a new NEXT_TRANSACTION_KEY from the given hash.
pub const fn next_transaction_key<'a>(hash: Hash) -> &'a [u8] {
    // Format the normal next tx path with the given hash
    format!("ledger::transactions::next({})", hash.to_str()).as_bytes()
}

/// Constructs a new
pub const fn transaction_with_hash_key<'a>(hash: Hash) -> &'a [u8] {
    // Format the normal transaction path with the given hash
    format!("ledger::transactions::transaction({})", hash.to_str()).as_bytes()
}
