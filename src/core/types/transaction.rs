use ed25519_dalek::Signature; // Import the edwards25519 digital signature library

use time::Tm; // Import the timestamp type from the time library

use num::bigint::BigUint; // Add support for large unsigned integers

use std::collections::HashMap; // Import the hash map module

use super::logs; // Import the logs module

use super::super::super::{common::address, crypto::hash}; // Import the hash & address modules

/// A transaction between two different addresses on the SummerCash network.
struct Transaction<'a> {
    /// The sender of the transaction
    sender: address::Address,
    /// The recipient of the transaction
    recipient: address::Address,
    /// The amount of finks sent along with the transaction
    value: BigUint,
    /// The data sent to the transaction recipient (i.e. contract call bytecode)
    payload: &'a [u8],
    /// The recipient's signature
    signature: Signature,
    /// The hashes of the transaction's parents
    parents: &'a [hash::Hash],
    /// The list of resolved parent receipts
    parent_receipts: HashMap<hash::Hash, Receipt<'a>>,
    /// The transaction's timestamp
    timestamp: Tm,
    /// The address of the deployed contract (if applicable)
    deployed_contract_address: address::Address,
    /// Whether or not this transaction creates a contract
    contract_creation: bool,
    /// Whether or not this transaction is the network genesis
    genesis: bool,
}

/// A receipt of a transaction's execution.
struct Receipt<'a> {
    /// Hash of state at transaction
    state_hash: hash::Hash,
    /// Logs emitted at run time
    logs: &'a [logs::Log<'a>],
}
