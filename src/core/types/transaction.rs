use ed25519_dalek::{Keypair, Signature}; // Import the edwards25519 digital signature library

use chrono; // Import time library

use num::bigint::BigUint; // Add support for large unsigned integers

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::receipt; // Import receipt types

use super::super::super::{common::address, crypto::blake2, crypto::hash}; // Import the hash & address modules

/// A transaction between two different addresses on the SummerCash network.
#[derive(Serialize, Deserialize)]
pub struct Transaction<'a> {
    /// The contents of the transaction
    #[serde(borrow)]
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
#[derive(Serialize, Deserialize)]
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
    #[serde(with = "serde_bytes")]
    payload: &'a [u8],
    /// The hashes of the transaction's parents
    parents: Vec<hash::Hash>,
    /// The list of resolved parent receipts
    parent_receipts: receipt::ReceiptMap<'a>,
    /// The transaction's timestamp
    timestamp: chrono::DateTime<chrono::Utc>,
}

/* BEGIN EXPORTED METHODS */

impl<'a> TransactionData<'a> {
    pub fn to_bytes(&'a self) -> &[u8] { return b"test"; }
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
        parents: Vec<hash::Hash>,
    ) -> Transaction<'a> {
        let transaction_data: TransactionData = TransactionData {
            nonce: nonce,                   // Set nonce
            sender: sender,                 // Set sender
            recipient: recipient,           // Set recipient
            value: value_finks,             // Set value (in finks)
            payload: payload,               // Set payload
            parents: parents,               // Set parents
            parent_receipts: None.unwrap(), // Set parent receipts
            timestamp: chrono::Utc::now(),  // Set timestamp
        }; // Initialize transaction data

        let mut transaction_data_bytes = vec![0; transaction_data.to_bytes().len()]; // Initialize transaction data buffer

        transaction_data_bytes.clone_from_slice(transaction_data.to_bytes()); // Clone into buffer

        Transaction {
            transaction_data: transaction_data, // Set transaction data
            hash: blake2::hash_slice(transaction_data_bytes.as_slice()),
            signature: None.unwrap(), // Set signature
            deployed_contract_address: None.unwrap(),
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
pub fn sign_transaction<'a>(keypair: Keypair, transaction: &'a mut Transaction) {
    let update
    transaction = &Transaction {
        transaction_data: TransactionData {
            nonce: transaction.transaction_data.nonce,   // Set nonce
            sender: transaction.transaction_data.sender, // Set sender
            recipient: transaction.transaction_data.recipient, // Set recipient
            value: transaction.transaction_data.value,   // Set value (in finks)
            payload: transaction.transaction_data.payload, // Set payload
            parents: transaction.transaction_data.parents, // Set parents
            parent_receipts: transaction.transaction_data.parent_receipts, // Set parent receipts
            timestamp: transaction.transaction_data.timestamp,                  // Set timestamp
        },
        hash: transaction.hash,                      // Set hash
        signature: keypair.sign(&*transaction.hash), // Set signature
        deployed_contract_address: transaction.deployed_contract_address,
        contract_creation: transaction.contract_creation, // Set does create contract
        genesis: transaction.genesis,                     // Set is genesis
    };
}

/* END EXPORTED METHODS */
