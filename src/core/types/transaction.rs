use ed25519_dalek::{Keypair, Signature}; // Import the edwards25519 digital signature library

use time::Tm; // Import the timestamp type from the time library

use num::bigint::BigUint; // Add support for large unsigned integers

use std::collections::HashMap; // Import the hash map module

use super::logs; // Import the logs module

use serde::ser::{Serialize, SerializeStruct, Serializer}; // Import serde

use super::super::super::{common::address, crypto::hash}; // Import the hash & address modules

/// A receipt of a transaction's execution.
pub struct Receipt<'a> {
    /// Hash of state at transaction
    state_hash: hash::Hash,
    /// Logs emitted at run time
    logs: &'a [logs::Log<'a>],
}

/// A transaction between two different addresses on the SummerCash network.
pub struct Transaction<'a> {
    /// The contents of the transaction
    transaction_data: TransactionData<'a>,
    /// The hash of the transaction
    hash: hash::Hash,
    /// The recipient's signature
    signature: Signature,
    /// The address of the deployed contract (if applicable)
    deployed_contract_address: address::Address,
    /// Whether or not this transaction creates a contract
    contract_creation: bool,
    /// Whether or not this transaction is the network genesis
    genesis: bool,
}

/// A container representing the contents of a transaction.
struct TransactionData<'a> {
    /// The index of the transaction in the sender's set of txs
    nonce: u128,
    /// The sender of the transaction
    sender: address::Address,
    /// The recipient of the transaction
    recipient: address::Address,
    /// The amount of finks sent along with the Transaction
    value: BigUint,
    /// The data sent to the transaction recipient (i.e. contract call bytecode)
    payload: &'a [u8],
    /// The hashes of the transaction's parents
    parents: &'a [hash::Hash],
    /// The list of resolved parent receipts
    parent_receipts: HashMap<hash::Hash, Receipt<'a>>,
    /// The transaction's timestamp
    timestamp: Tm,
}

/* BEGIN EXPORTED METHODS */

/// Implement a set of serialization helper methods.
impl Receipt {
    /// Serialize the receipt to a byte slice.
    pub fn serialize<S>(&self, serializer: S) -> Result<S:: Ok, S::Error> where S: Serializer, {
        let mut state = serializer.serialize_struct("Receipt", 2)?; // Serialize receipt struct

        let serialized_logs: Vec<&[u8]>; // Initialize serialized logs buffer

        for log in self.logs { // Iterate through logs
            serialized_logs.push(log.serialize()?); // Serialize log
        }

        state.serialize_field("state_hash", &*self.state_hash); // Serialize state hash field
        state.serialize_field("logs", &self.logs[0]); // Serialize logs field
    }
}

impl TransactionData {
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TransactionData", 8)?; // Serialize TransactionData struct

        let parents: Vec<[u8; hash::HASH_SIZE]>; // Initialize parents buffer

        for parent in self.parents {
            // Iterate through parents
            parents.push(**parent); // Append normalized parent to list of normalized parents
        }

        state.serialize_field("nonce", &self.nonce)?; // Serialize nonce field
        state.serialize_field("sender", &*self.sender)?; // Serialize sender field
        state.serialize_field("recipient", &*self.recipient)?; // Serialize recipient field
        state.serialize_field("value", &self.value.to_bytes_le())?; // Serialize value field
        state.serialize_field("payload", &self.payload)?; // Serialize payload field
        state.serialize_field("parents", &parents)?; // Serialize parents field
        state.serialize_field("parent_receipts", &self.parent_receipts); // Serialize parent receipts field
        state.serialize_field("timestamp", &self.timestamp); // Serializse timestamp field
    }
    pub fn to_bytes(&self) -> &[u8] {
        return;
    }
}

/// Implement a set of transaction helper methods.
impl<'a> Transaction<'a> {
    /// Initialize a new transaction instance from a given set of parameters.
    ///
    /// # Example TODO: Example
    pub fn new(
        nonce: u128,
        sender: address::Address,
        recipient: address::Address,
        value_finks: BigUint,
        payload: &'a [u8],
        parents: &'a [hash::Hash],
    ) -> Transaction<'a> {
        Transaction {
            transaction_data: TransactionData {
                nonce: nonce,                   // Set nonce
                sender: sender,                 // Set sender
                recipient: recipient,           // Set recipient
                value: value_finks,             // Set value (in finks)
                payload: payload,               // Set payload
                parents: parents,               // Set parents
                parent_receipts: None?, // Set parent receipts
                timestamp: time::now_utc(),     // Set timestamp
            }, // Set transaction data
            hash: crypto::blake2::hash_slice(),
            signature: None?, // Set signature
            deployed_contract_address: None?,
            contract_creation: false, // Set does create contract
            genesis: false,           // Set is genesis
        }
    }
}

/// Sign a given transaction with the provided ed25519 keypair.
///
/// # Example
///
/// ```
/// extern crate num; // Link num library
///
/// use num::traits::FromPrimitive; // Allow overloading of from_i64()
///
/// use num::bigint::BigUint; // Add support for large unsigned integers
///
/// use summercash::core::types::transaction; // Import the transaction library
/// use summercash::{common::address, crypto::hash}; // Import the address library
///
/// let sender = address::Address::from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"); // Decode sender address from hex
/// let recipient = address::Address::from_str("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"); // Decode recipient address from hex
///
/// let some_parent_hash = hash::Hash::from_str("928b20366943e2afd11ebc0eae2e53a93bf177a4fcf35bcc64d503704e65e202"); // Decode parent hash from hex
///
/// let transaction = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0), b"test transaction payload", [some_parent_hash]); // Initialize transaction
/// ```
pub fn sign_transaction<'a>(keypair: Keypair, transaction: &Transaction) -> &Transaction {
    &Transaction {
        transaction_data: TransactionData {
            nonce: transaction.transaction_data.nonce,   // Set nonce
            sender: transaction.transaction_data.sender, // Set sender
            recipient: transaction.transaction_data.recipient, // Set recipient
            value: transaction.transaction_data.value,   // Set value (in finks)
            payload: transaction.transaction_data.payload, // Set payload
            parents: transaction.transaction_data.parents, // Set parents
            parent_receipts: transaction.transaction_data.parent_receipts, // Set parent receipts
            timestamp: time::now_utc(),                  // Set timestamp
        },
        hash: transaction.hash,                      // Set hash
        signature: keypair.sign(&*transaction.hash), // Set signature
        deployed_contract_address: transaction.deployed_contract_address,
        contract_creation: transaction.contract_creation, // Set does create contract
        genesis: transaction.genesis,                     // Set is genesis
    }
}

/* END EXPORTED METHODS */
