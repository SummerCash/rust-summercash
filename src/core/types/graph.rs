use super::transaction; // Import transaction types
use super::state; // Import state module

use super::super::super::{crypto::hash}; // Import address types

/// A node in any particular state-entry/transaction-based DAG.
pub struct Node<'a> {
    /// The transaction associated with a given node
    pub transaction: transaction::Transaction<'a>,
    /// The state entry associated with a given node
    pub state_entry: Option<state::state_entry::Entry>,
    /// The hash of the transaction associated with a given node
    pub hash: hash::Hash,
}

/// A generic DAG used to store state entries, as well as transactions.
pub struct Graph<'a> {
    /// A list of nodes in the graph
    pub nodes: Vec<Node<'a>>,
}

/// Implement a set of node helper methods.
impl<'a> Node<'a> {
    /// Initialize a new node with a given state entry and transaction.
    /// 
    /// # Example
    ///
    /// ```
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::{transaction, graph}; // Import the transaction, graph libraries
    /// use summercash::{common::address, crypto::hash}; // Import the address, hash libraries
    ///
    /// let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    /// 
    /// let node = graph::Node::new(tx, None); // Initialize node
    /// ```
    pub fn new(transaction: transaction::Transaction<'a>, state_entry: Option<state::state_entry::Entry>) -> Node {
        let transaction_hash = transaction.hash.clone(); // Clone transaction hash

        Node {
            transaction: transaction, // Set transaction
            state_entry: state_entry, // Set state entry
            hash: transaction_hash, // Set transaction hash
        } // Return initialized node
    }
}

/// Implement a set of graph helper methods.
impl<'a> Graph<'a> {
    /// Initialize a new graph instance.
    /// 
    /// # Example
    ///
    /// ```
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::{transaction, graph}; // Import the transaction, graph libraries
    /// use summercash::{common::address, crypto::hash}; // Import the address, hash libraries
    ///
    /// let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    /// 
    /// let dag = graph::Graph::new(tx); // Initialize graph
    /// ```
    pub fn new(root_transaction: transaction::Transaction<'a>) -> Graph<'a> {
        let root_transaction_hash = root_transaction.hash.clone(); // Clone transaction hash

        Graph{
            nodes: vec![Node{transaction: root_transaction, state_entry: None, hash: root_transaction_hash}], // Set nodes
        } // Return initialized graph
    }
}