use super::super::crypto::hash::Hash;

use libp2p::kad::record::Key;

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
