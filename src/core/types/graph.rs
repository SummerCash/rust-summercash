use super::state; // Import state module
use super::transaction; // Import transaction types

use sled; // Import sled database

use std::collections; // Import collections module

use serde::{Deserialize, Serialize}; // Import serde serialization
use bincode; // Import serde bincode

use super::super::super::{common::address, common::io, crypto::hash}; // Import address, hash types

/// An error encountered while signing a tx.
#[derive(Debug, Fail)]
pub enum OperationError {
    #[fail(
        display = "encountered an error while attempting lookup for key {}: {}",
        key, error
    )]
    NoLookupResults {
        key: String,   // The queried key
        error: String, // The error
    },
    #[fail(
        display = "failed to execute transaction with hash {}; state has already been resolved",
        transaction_hash
    )]
    AlreadyExecuted {
        transaction_hash: String, // The transaction hash
    },
}

/// A node in any particular state-entry/transaction-based DAG.
#[derive(Serialize, Deserialize, Clone)]
pub struct Node<'a> {
    /// The transaction associated with a given node
    #[serde(borrow)]
    pub transaction: transaction::Transaction<'a>,
    /// The state entry associated with a given node
    pub state_entry: Option<state::Entry>,
    /// The hash of the transaction associated with a given node
    pub hash: hash::Hash,
}

/// A generic DAG used to store state entries, as well as transactions.
pub struct Graph<'a> {
    /// A list of nodes in the graph
    pub nodes: Vec<Node<'a>>,
    /// A list of routes to addresses in the graph (by usize index)
    hash_routes: collections::HashMap<address::Address, usize>,
    /// A list of children for a given node in the graph
    node_children: collections::HashMap<hash::Hash, Vec<hash::Hash>>,
    /// A persisted database instance
    db: Option<sled::Db>,
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
    pub fn new(
        transaction: transaction::Transaction<'a>,
        state_entry: Option<state::Entry>,
    ) -> Node {
        let transaction_hash = transaction.hash.clone(); // Clone transaction hash

        Node {
            transaction: transaction, // Set transaction
            state_entry: state_entry, // Set state entry
            hash: transaction_hash,   // Set transaction hash
        } // Return initialized node
    }

    /// Verify the contents of a given node (i.e. hashes match).
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
    ///
    /// let is_valid = node.verify_contents(); // False, since state entry is None
    /// ```
    pub fn verify_contents(&self) -> bool {
        self.transaction.hash == self.hash // Return hashes are equivalent
    }

    /// Perform all possible verification tests (both to check that values exist, and that they are indeed valid; e.g. validate signatures).
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
    ///
    /// let is_valid = node.perform_validity_checks(); // False, since state entry is None TODO: Add more checks
    /// ```
    pub fn perform_validity_checks(&self) -> bool {
        let contents_valid = self.verify_contents(); // Verify contents of self

        match contents_valid {
            true => self.transaction.verify_signature(),
            false => false,
        }
    }

    /// Serialize a graph node instance to vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap() // Serialize self
    }

    /// Deserialize a graph node instance from a vector.
    pub fn from_bytes(b: &'a [u8]) -> Node<'a> {
        bincode::deserialize(b).unwrap()
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
        let root_transaction_state_entry = root_transaction.execute(None); // Execute root transaction

        let mut hash_routes = collections::HashMap::new(); // Initialize address routes map
        hash_routes.insert(root_transaction_hash, 0); // Set root transaction route

        Graph {
            nodes: vec![Node {
                transaction: root_transaction,                   // Set transaction
                state_entry: Some(root_transaction_state_entry), // Set state entry
                hash: root_transaction_hash,                     // Set hash
            }], // Set nodes
            hash_routes: hash_routes, // Set address routes
            node_children: collections::HashMap::new(), // Set node children
            db: None, // Set db
        } // Return initialized dag
    }

    /// Push a new item to the graph.
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
    /// let tx2 = transaction::Transaction::new(1, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize second transaction
    ///
    /// let mut dag = graph::Graph::new(tx); // Initialize graph
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    /// ```
    pub fn push(
        &mut self,
        transaction: transaction::Transaction<'a>,
        state_entry: Option<state::Entry>,
    ) -> usize {
        let transaction_hash = transaction.hash.clone(); // Clone transaction hash value
        let transaction_parents = transaction.transaction_data.parents.clone(); // Clone transaction parents

        self.nodes.push(Node::new(transaction, state_entry)); // Push node to graph
        self.hash_routes
            .insert(transaction_hash, self.nodes.len() - 1); // Set route to node

        for parent in transaction_parents {
            // Iterate through transaction parents
            if !self.node_children.contains_key(&parent) {
                // Check parent does not already exist in list of child routes from parent
                self.node_children.insert(parent, vec![transaction_hash]); // Set transaction hash as child of parent in graph

                break; // Break loop
            }

            self.node_children
                .get_mut(&parent)
                .unwrap()
                .push(transaction_hash); // Add transaction as child of parent in graph
        }

        self.nodes.len() - 1 // Return index of transaction
    }

    /// Update an item in the graph.
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
    /// let tx2 = transaction::Transaction::new(1, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize second transaction
    ///
    /// let mut dag = graph::Graph::new(tx); // Initialize graph
    ///
    /// dag.update(0, tx2, None); // Update transaction in DAG
    /// ```
    pub fn update(
        &mut self,
        index: usize,
        transaction: transaction::Transaction<'a>,
        state_entry: Option<state::Entry>,
    ) {
        self.nodes[index] = Node::new(transaction, state_entry); // Set node in graph
    }

    /// Get a reference to the node at a given index.
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
    /// let tx2 = transaction::Transaction::new(1, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize second transaction
    ///
    /// let mut dag = graph::Graph::new(tx); // Initialize graph
    ///
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    /// let node = dag.get(index_of_transaction); // Get a reference to the corresponding node
    /// ```
    pub fn get(&self, index: usize) -> &'a Node {
        &self.nodes[index] // Return node
    }

    /// Get a reference to a node with the given hash.
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
    /// let tx2 = transaction::Transaction::new(1, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize second transaction
    /// let tx2_hash = tx2.hash.clone(); // Clone transaction 2 hash
    ///
    /// let mut dag = graph::Graph::new(tx); // Initialize graph
    ///
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    /// let node = dag.get_with_hash(tx2_hash); // Get a reference to the corresponding node
    /// ```
    pub fn get_with_hash(&self, hash: hash::Hash) -> Result<&'a Node, OperationError> {
        if self.hash_routes.contains_key(&hash) {
            // Check hash route to node with hash
            Ok(&self.nodes[*self.hash_routes.get(&hash).unwrap()]) // Return node
        } else {
            Err(OperationError::NoLookupResults {
                key: hash.to_str(),                         // Set key
                error: "no route to node found".to_owned(), // Set error
            }) // Return error in result
        }
    }

    /// Read a graph instance from the disk.
    pub fn read_from_disk(&self) -> Graph<'a> {
        let db = sled::Db::start_default(io::db_dir()).unwrap(); // Open database

        let mut graph = Graph{
            nodes: vec![], // Set nodes
            hash_routes: collections::hash_map::HashMap::new(), // Set address routes
            node_children: collections::hash_map::HashMap::new(), // Set node children
            db: None, // Set db to none until we initialize our graph
        }; // Initialize graph instance

        let mut iter = db.scan(b"0"); // Get iterator (start at genesis transaction)

        loop {
            let current_node: Node<'a>; // Initialize current node buffer

            match iter.next() { // Handle different next values
                // A value exists, set current node to deserialized next value
                Some(val) => {
                    match val {
                        // No error occurred, set current node
                        Ok(val) => current_node = Node::from_bytes(&val.1),
                        // An error occurred, skip over this node
                        _ => continue,
                    }
                },
                // No value exists, break
                None => break,
            };

            graph.push(current_node.transaction, current_node.state_entry); // Add node to graph
        }

        graph.db = Some(db); // Set graph db

        graph // Return initialized graph
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    use num::bigint::BigUint; // Add support for large unsigned integers
    use num::traits::FromPrimitive; // Allow overloading of from_i64()
    use rand::rngs::OsRng; // Import the os's rng

    use super::*; // Import names from parent module

    #[test]
    fn test_new() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
        let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair

        let root_tx = transaction::Transaction::new(
            0,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize root transaction

        let dag: Graph = Graph::new(root_tx); // Initialize graph

        assert_eq!(
            dag.nodes[0].transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure transaction payload retained
    }

    #[test]
    fn test_push() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
        let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair

        let root_tx = transaction::Transaction::new(
            0,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize root transaction
        let tx_2 = transaction::Transaction::new(
            1,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize second transaction

        let mut dag: Graph = Graph::new(root_tx); // Initialize graph

        let node_index: usize = dag.push(tx_2, None); // Push second transaction

        assert_eq!(node_index, 1); // Ensure is second transaction in DAG
    }

    #[test]
    fn test_update() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
        let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair

        let root_tx = transaction::Transaction::new(
            0,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize root transaction
        let tx_2 = transaction::Transaction::new(
            1,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload 2",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize second transaction

        let mut dag: Graph = Graph::new(root_tx); // Initialize graph

        dag.update(0, tx_2, None); // Update root transaction

        assert_eq!(
            dag.get(0).transaction.transaction_data.payload,
            b"test transaction payload 2"
        ); // Ensure has updated transaction
    }

    #[test]
    fn test_get() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
        let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair

        let root_tx = transaction::Transaction::new(
            0,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize root transaction

        let dag: Graph = Graph::new(root_tx); // Initialize graph

        let found_root_tx = dag.get(0); // Get root tx

        assert_eq!(
            found_root_tx.transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure is same transaction
    }

    #[test]
    fn test_get_with_hash() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
        let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair

        let root_tx = transaction::Transaction::new(
            0,
            sender,
            recipient,
            BigUint::from_i64(0).unwrap(),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize root transaction
        let root_tx_hash = root_tx.hash.clone(); // Clone root tx hash

        let dag: Graph = Graph::new(root_tx); // Initialize graph

        let found_root_tx = dag.get_with_hash(root_tx_hash).unwrap(); // Get root tx

        assert_eq!(
            found_root_tx.transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure is same transaction
    }
}
