use super::state::{self, Entry}; // Import state module
use super::transaction; // Import transaction types

use num::{bigint::BigUint, Zero};
use std::collections; // Import collections, io modules

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::{
    common::{address::Address, io},
    crypto::hash::{self, Hash},
}; // Import address, hash types

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
pub struct Node {
    /// The transaction associated with a given node
    pub transaction: transaction::Transaction,
    /// The state entry associated with a given node
    pub state_entry: Option<state::Entry>,
    /// The hash of the transaction associated with a given node
    pub hash: hash::Hash,
}

/// A generic DAG used to store state entries, as well as transactions.
#[derive(Clone)]
pub struct Graph {
    /// A list of nodes in the graph
    pub nodes: Vec<Node>,
    /// A list of routes to addresses in the graph (by usize index)
    pub hash_routes: collections::HashMap<hash::Hash, usize>,
    /// A list of children for a given node in the graph
    pub node_children: collections::HashMap<hash::Hash, Vec<hash::Hash>>,
    /// A persisted database instance
    db: Option<sled::Db>,
}

/// Implement a set of node helper methods.
impl Node {
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    pub fn new(transaction: transaction::Transaction, state_entry: Option<state::Entry>) -> Node {
        let transaction_hash = transaction.hash; // Clone transaction hash

        Node {
            transaction,            // Set transaction
            state_entry,            // Set state entry
            hash: transaction_hash, // Set transaction hash
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let is_valid = node.perform_validity_checks(); // False, since state entry is None
    /// ```
    pub fn perform_validity_checks(&self) -> bool {
        self.verify_contents() && self.transaction.verify_signature() // Verify contents of self
    }

    /// Serialize a graph node instance to vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap() // Serialize self
    }

    /// Deserialize a graph node instance from a vector.
    pub fn from_bytes(b: &[u8]) -> Node {
        bincode::deserialize(b).unwrap()
    }
}

/// We'll want to make sure everything has been closed before the Graph can be deallocated.
impl Drop for Graph {
    /// Deallocates the graph.
    fn drop(&mut self) {
        // Save the graph
        self.write_to_disk().unwrap();

        // Deallocate the db
        self.db.take();
    }
}

/// Implement a set of graph helper methods.
impl Graph {
    /// Initialize a new graph instance.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    ///
    /// let dag = graph::Graph::new(tx, "olympia"); // Initialize graph
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn new(root_transaction: transaction::Transaction, network_name: &str) -> Graph {
        Graph::new_with_db_path(root_transaction, &io::format_db_dir(network_name))
        // Return initialized graph
    }

    /// Initialize a new graph instance, and store the corresponding db in db_path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    /// extern crate path_clean; // Link path clean library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use path_clean; // Import path clean module
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::{transaction, graph}; // Import the transaction, graph libraries
    /// use summercash::{common::{address, io}, crypto::hash}; // Import the address, hash libraries
    ///
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    ///
    /// let dag = graph::Graph::new_with_db_path(tx, format!(io::data_dir(), "/data")); // Initialize graph
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn new_with_db_path(root_transaction: transaction::Transaction, db_path: &str) -> Graph {
        let root_transaction_hash = root_transaction.hash; // Clone transaction hash
        let root_transaction_state_entry = root_transaction.execute(None); // Execute root transaction

        let mut hash_routes = collections::HashMap::new(); // Initialize address routes map
        hash_routes.insert(root_transaction_hash, 0); // Set root transaction route

        Graph {
            nodes: vec![Node {
                transaction: root_transaction,                   // Set transaction
                state_entry: Some(root_transaction_state_entry), // Set state entry
                hash: root_transaction_hash,                     // Set hash
            }], // Set nodes
            hash_routes,                                // Set address routes
            node_children: collections::HashMap::new(), // Set node children
            db: Some(sled::open(db_path).unwrap()),     // Set db
        } // Return initialized dag
    }

    /// Push a new item to the graph.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let mut dag = graph::Graph::new(tx, "olympia"); // Initialize graph
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    ///
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn push(
        &mut self,
        transaction: transaction::Transaction,
        state_entry: Option<state::Entry>,
    ) -> usize {
        let transaction_hash = transaction.hash; // Clone transaction hash value
        let transaction_parents = transaction.transaction_data.parents.clone(); // Clone transaction parents

        self.nodes.push(Node::new(transaction, state_entry)); // Push node to graph
        self.hash_routes
            .insert(transaction_hash, self.nodes.len() - 1); // Set route to node

        for parent in transaction_parents {
            self.node_children
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(transaction_hash); // Add transaction as child of parent in graph
        }

        self.nodes.len() - 1 // Return index of transaction
    }

    /// Push a new node to the graph.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to push to the graph
    pub fn add(&mut self, node: Node) -> usize {
        // Put the node in the graph
        self.push(node.transaction, node.state_entry)
    }

    /// Purges the contents of each of the nodes in the in-memory graph.
    pub fn purge(&mut self) {
        // Go through each of the nodes & manually purge
        for i in 0..self.nodes.len() {
            // Reset the state contents of the nodes
            self.nodes[i].state_entry = None;
        }
    }

    /// Update an item in the graph.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let mut dag = graph::Graph::new(tx, "olympia"); // Initialize graph
    /// dag.update(0, tx2, None); // Update transaction in DAG
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn update(
        &mut self,
        index: usize,
        transaction: transaction::Transaction,
        state_entry: Option<state::Entry>,
    ) {
        self.nodes[index] = Node::new(transaction, state_entry); // Set node in graph
    }

    /// Gets a copy of the node at a given index without modifying the graph's state.
    pub fn get_pure(&self, index: usize) -> Result<Option<Node>, sled::Error> {
        // Get a copy of the node
        let mut node = self.nodes[index].clone();

        // Fill the node's state
        match node.state_entry {
            Some(_) => Ok(Some(node)),
            None => {
                // Check db opened
                if let Some(db) = &self.db {
                    let node_query_result = db.get(index.to_string().as_bytes())?; // Query db for node

                    // Handle different result types
                    match node_query_result {
                        // Success!
                        Some(bytes_encoded_node) => {
                            let deserialized_node: Node =
                                Node::from_bytes(&bytes_encoded_node.to_vec()[..]); // Deserialize node
                            node.state_entry = deserialized_node.state_entry; // Set state entry

                            return Ok(Some(node)); // Return deserialized node
                        }
                        // Couldn't find node in db
                        None => return Ok(Some(node)),
                    };
                }

                Ok(Some(node)) // Return node, since we can't do a full load anyway
            }
        }
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let mut dag = graph::Graph::new(tx, "olympia"); // Initialize graph
    ///
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    /// let node = dag.get(index_of_transaction); // Get a reference to the corresponding node
    ///
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn get(&mut self, index: usize) -> Result<Option<&Node>, sled::Error> {
        let node = &mut self.nodes[index]; // Get ref to node

        // Check was partially or fully loaded
        match node.state_entry {
            // Loaded fully
            Some(_) => Ok(Some(node)),
            // Loaded partially
            None => {
                // Check db opened
                if let Some(db) = &self.db {
                    let node_query_result = db.get(index.to_string().as_bytes())?; // Query db for node

                    // Handle different result types
                    match node_query_result {
                        // Success!
                        Some(bytes_encoded_node) => {
                            let deserialized_node: Node =
                                Node::from_bytes(&bytes_encoded_node.to_vec()[..]); // Deserialize node
                            node.state_entry = deserialized_node.state_entry; // Set state entry

                            return Ok(Some(node)); // Return deserialized node
                        }
                        // Couldn't find node in db
                        None => return Ok(Some(node)),
                    };
                }

                Ok(Some(node)) // Return node, since we can't do a full load anyway
            }
        }
    }

    /// Get a reference to a node with the given hash.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
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
    /// let mut dag = graph::Graph::new(tx, "olympia"); // Initialize graph
    ///
    /// let index_of_transaction = dag.push(tx2, None); // Add transaction to DAG
    /// let node = dag.get_with_hash(tx2_hash); // Get a reference to the corresponding node
    ///
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn get_with_hash(&self, hash: hash::Hash) -> Result<&Node, OperationError> {
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

    /// Get the last recorded balance of the provided account.
    ///
    /// # Arguments
    ///
    /// * `account` - The account of which the balance should be determined
    pub fn get_balance_of_account(&self, account: &Address) -> BigUint {
        // Get a non-empty state in which the account's balance might reside
        if let Some(head) = self.obtain_executed_head() {
            // The state must have a state entry, since it was marked as "executed"
            if let Some(state) = head.state_entry {
                // Get the balance of the account
                return state
                    .data
                    .balances
                    .get(&account.to_str())
                    .unwrap_or(&BigUint::zero())
                    .clone();
            }
        }

        // The account must have a balance of zero, since it hasn't been included in the DAG yet
        BigUint::zero()
    }

    /// Gets the number of finks issued in the genesis block.
    pub fn overall_issuance(&self) -> BigUint {
        // Get the genesis transaction
        if let Ok(Some(genesis)) = self.get_pure(0) {
            // Try to get the state, and, by extension, all of the balances at the genesis
            // transaction
            if let Some(state) = genesis.state_entry {
                // We're going to sum up the coins issued to each address; make a var to store this
                let mut total_issuance = BigUint::zero();

                // Collect the balances of each address
                for balance in state.data.balances.values().into_iter() {
                    total_issuance += balance;
                }

                return total_issuance;
            }
        }

        BigUint::zero()
    }

    /// Read the entirety of a persisted graph, or just state entry headers.
    fn read_some_from_disk(read_all: bool, network: &str) -> Self {
        // Read the database
        Self::read_some_from_disk_with_data_dir(read_all, &io::format_db_dir(network))
    }

    /// Read the entirety of a persisted graph, or just state entry headers.
    fn read_some_from_disk_with_data_dir(read_all: bool, directory: &str) -> Graph {
        let db = sled::open(directory).unwrap(); // Open database

        let mut nodes: Vec<Node> = vec![]; // Empty vector
        let mut hash_routes: collections::hash_map::HashMap<hash::Hash, usize> =
            collections::hash_map::HashMap::new(); // Initialize hash routes map buffer
        let mut node_children: collections::hash_map::HashMap<hash::Hash, Vec<hash::Hash>> =
            collections::hash_map::HashMap::new(); // Initialize child routes map buffer

        let iter = db.iter(); // Get iterator (start at genesis transaction)

        iter.for_each(|key_val_pair| {
            if let Ok(val) = key_val_pair {
                let mut current_node: Node = Node::from_bytes(&val.1.to_vec()[..]); // Deserialize node

                if !read_all {
                    // Check should disregard state data
                    current_node.state_entry = None; // Set state entry to nil
                }

                hash_routes.insert(current_node.hash.clone(), nodes.len()); // Insert route to node

                for parent in current_node.transaction.transaction_data.clone().parents {
                    node_children
                        .entry(parent)
                        .or_insert_with(|| vec![])
                        .push(current_node.hash.clone());
                }

                nodes.push(current_node); // Add current node to nodes list
            }
        }); // Add nodes to graph vars

        Graph {
            nodes,         // Set nodes
            hash_routes,   // Set address routes
            node_children, // Set node children
            db: Some(db),  // Set db to none until we initialize our graph
        } // Return initialized graph
    }

    /// Read the transactions--but not state data--in a graph from the disk.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use summercash::core::types::graph; // Import the graph module
    ///
    /// let dag: graph::Graph = graph::Graph::read_partial_from_disk(); // Read txs, but not state data from disk
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn read_partial_from_disk(network_name: &str) -> Graph {
        Graph::read_some_from_disk(false, network_name) // Read just transaction headers
    }

    /// Read the transactions--but not state data--in a graph from the disk, where the database is located in the given data_dir.
    pub fn read_partial_from_disk_with_data_dir(data_dir: &str, network_name: &str) -> Self {
        // Read just transaction headers
        Graph::read_some_from_disk_with_data_dir(
            false,
            &format!("{}/db/{}", data_dir, network_name),
        )
    }

    /// Read a graph instance from the disk.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use summercash::core::types::graph; // Import the graph module
    ///
    /// let dag: graph::Graph = graph::Graph::read_from_disk(); // Read graph from disk
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn read_from_disk(network_name: &str) -> Graph {
        Graph::read_some_from_disk(true, network_name) // Read entirety of graph
    }

    /// Write a graph instance to the disk, and close the associated database instance.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    ///
    /// let dag: graph::Graph = graph::Graph::new(tx, "olympia"); // Initialize graph
    /// assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    /// ```
    pub fn write_to_disk(&self) -> Result<(), sled::Error> {
        // Get database instance
        if let Some(db) = &self.db {
            // Iterate through nodes
            for (i, node) in self.nodes.iter().rev().enumerate() {
                // Only continue with the persistence process if the nodes haven't already been
                // saved to the database
                if db.contains_key(i.to_string().as_bytes())? {
                    break;
                }

                // Save the node in the database
                db.insert(i.to_string().as_bytes(), node.to_bytes())?;
            }

            db.flush()?; // Close db
        } else {
            return Err(sled::Error::Unsupported(
                "could not open database".to_owned(),
            )); // Return error
        }

        Ok(()) // Done!
    }

    /// Gets a reference to the head node in the graph--that which contains a resolved state.
    pub fn obtain_executed_head(&self) -> Option<Node> {
        // Start with the last node added to the graph, and move backwards.
        // We're looking for a node that contains a non-empty state
        for i in (0..self.nodes.len()).rev() {
            // Check that we can find a valid state entry for this node
            if let Ok(Some(node)) = self.get_pure(i) {
                if node.state_entry.is_some() {
                    // Return the state
                    return Some(node);
                }
            }
        }

        // We couldn't find a state
        None
    }

    /// Removes the head transaction, and rolls back its direct parents. If there is no head, no computation occurs.
    pub fn rollback_head(&mut self) {
        // Remove the head from the nodes list
        if let Some(removed_node) = self.nodes.pop() {
            // Remove the route to the transaction by its hash
            self.hash_routes.remove(&removed_node.hash);

            // Remove the child from each parent
            for parent in removed_node.transaction.transaction_data.parents {
                // Remove the child from the parent, if it has any children it can remember
                if let Some(children) = self.node_children.get_mut(&parent) {
                    // Remove the child from the parent's memory
                    children.pop();
                }

                // If the parent exists, remove the state, since we gotta roll back
                if let Some(parent_node) = self.hash_routes.get(&parent) {
                    // Reset the node's state
                    self.nodes[*parent_node].state_entry = None;
                }
            }
        }
    }

    /// Executes each of the parent nodes with the provided hashes, merging their resultant states. NOTE: This method is pure.
    pub fn resolve_parent_nodes(
        &self,
        parents: Vec<Hash>,
    ) -> Result<(Entry, Vec<(Hash, Entry)>), sled::Error> {
        // The set of states resulting from the execution of each of the provided nodes
        let mut merged_state: Entry = Default::default();
        let mut individual_states: Vec<(Hash, Entry)> = Vec::new();

        // Execute each of the provided nodes, and collect a state entry describing such an execution
        for node_hash in parents {
            // Get the index of this node
            let index = if let Some(i) = self.hash_routes.get(&node_hash) {
                *i
            } else {
                continue;
            };

            // Get the actual node at the index
            let node = if let Ok(Some(n)) = self.get_pure(index) {
                n
            } else {
                continue;
            };

            // If the node already has a state entry, we should be able to continue on without executing it
            if let Some(entry) = node.state_entry.clone() {
                // Just use the entry that the node already has
                individual_states.push((node_hash, entry.clone()));
                merged_state = state::merge_entries(vec![merged_state, entry]);

                continue;
            }

            // If the transaction doesn't have any parents, we can just execute it without any params
            if node.transaction.transaction_data.parents.is_empty() {
                // Execute the transaction
                individual_states.push((node_hash, node.transaction.execute(None)));
                merged_state = state::merge_entries(vec![
                    merged_state,
                    individual_states[individual_states.len() - 1].1.clone(),
                ]);

                continue;
            }

            // Try to execute the parents of this transaction. If this succeeds, we can resolve the immediate state.
            if let Ok(prev_state) =
                self.resolve_parent_nodes(node.transaction.transaction_data.parents.clone())
            {
                // Execute the transaction, and merge it back on to the overall state
                individual_states.push((node_hash, node.transaction.execute(Some(prev_state.0))));
                merged_state = state::merge_entries(vec![
                    merged_state,
                    individual_states[individual_states.len() - 1].1.clone(),
                ]);
            }
        }

        // Combine each of the states that the parent nodes produced into one gigantic state entry
        Ok((merged_state, individual_states))
    }

    /// Resolve states for all parent nodes, direct or indirect. NOTE: This method is not pure.
    pub fn execute_parent_nodes(
        &mut self,
        child_index: usize,
    ) -> Result<state::Entry, sled::Error> {
        // Get node
        if let Some(node) = self.get(child_index)? {
            let mut parent_entries: Vec<state::Entry> = vec![]; // Initialize parent entries vec

            for parent in node.transaction.transaction_data.parents.clone() {
                // Iterate through node parents
                if let Some(index) = self.clone().hash_routes.get(&parent) {
                    // Get index of parent
                    if let Some(node) = self.nodes.get(*index) {
                        // If the node has already been executed, we don't need to do the heavy lifting
                        if let Some(state_entry) = node.state_entry.clone() {
                            // Check already has state entry
                            parent_entries.push(state_entry); // Add state entry to parent entries vec

                            continue; // Continue
                        }
                    }

                    if self.nodes[*index]
                        .transaction
                        .transaction_data
                        .parents
                        .is_empty()
                    {
                        // Execute the node, without any prior metadata, since the transaction is the root
                        let executed = self.nodes[*index].transaction.execute(None);

                        // Keep the executed state in mind for later computation
                        self.nodes[*index].state_entry = Some(executed.clone());
                        parent_entries.push(executed);

                        continue;
                    }

                    if let Ok(prev_state_entry) = self.execute_parent_nodes(*index) {
                        // Execute the node with the given prior metadata / state
                        let executed = self.nodes[*index]
                            .transaction
                            .execute(Some(prev_state_entry));

                        // Store the calculated state in the tree, and in the collection vector
                        self.nodes[*index].state_entry = Some(executed.clone());
                        parent_entries.push(self.nodes[*index].state_entry.clone().unwrap());
                    }
                }
            }

            Ok(state::merge_entries(parent_entries)) // Return merged entries
        } else {
            Err(sled::Error::CollectionNotFound(
                (&[child_index as u8]).into(),
            )) // Return error
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rand::Rng; // Import rand
    use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    use num::bigint::BigUint; // Add support for large unsigned integers
    use num::traits::FromPrimitive; // Allow overloading of from_i64()
    use rand; // Import the rand module
    use rand::rngs::OsRng; // Import the os's rng

    use path_clean; // Import path clean module

    use super::super::super::super::common::address; // Import address module

    use super::*; // Import names from parent module

    #[test]
    fn test_new() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng(); // Generate source of randomness

        let rand: u16 = rng.gen(); // Generate random number

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

        let dag: Graph = Graph::new_with_db_path(
            root_tx,
            &path_clean::clean(&format!("{}/.tests/{}", io::db_dir(), rand.to_string())),
        ); // Initialize graph

        assert_eq!(
            dag.nodes[0].transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure transaction payload retained

        assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    }

    #[test]
    fn test_push() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng(); // Generate source of randomness

        let rand: u16 = rng.gen(); // Generate random number

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

        let mut dag: Graph = Graph::new_with_db_path(
            root_tx,
            &path_clean::clean(&format!("{}/.tests/{}", io::db_dir(), rand.to_string())),
        ); // Initialize graph

        let node_index: usize = dag.push(tx_2, None); // Push second transaction

        assert_eq!(node_index, 1); // Ensure is second transaction in DAG

        assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    }

    #[test]
    fn test_update() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng(); // Generate source of randomness

        let rand: u16 = rng.gen(); // Generate random number

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

        let mut dag: Graph = Graph::new_with_db_path(
            root_tx,
            &path_clean::clean(&format!("{}/.tests/{}", io::db_dir(), rand.to_string())),
        ); // Initialize graph

        dag.update(0, tx_2, None); // Update root transaction

        assert_eq!(
            dag.get(0)
                .unwrap()
                .unwrap()
                .transaction
                .transaction_data
                .payload,
            b"test transaction payload 2"
        ); // Ensure has updated transaction

        assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    }

    #[test]
    fn test_get() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng(); // Generate source of randomness

        let rand: u16 = rng.gen(); // Generate random number

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

        let mut dag: Graph = Graph::new_with_db_path(
            root_tx,
            &path_clean::clean(&format!("{}/.tests/{}", io::db_dir(), rand.to_string())),
        ); // Initialize graph

        let found_root_tx = dag.get(0).unwrap().unwrap(); // Get root tx

        assert_eq!(
            found_root_tx.transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure is same transaction

        assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    }

    #[test]
    fn test_get_with_hash() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng(); // Generate source of randomness

        let rand: u16 = rng.gen(); // Generate random number

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

        let dag: Graph = Graph::new_with_db_path(
            root_tx,
            &path_clean::clean(&format!("{}/.tests/{}", io::db_dir(), rand.to_string())),
        ); // Initialize graph

        let found_root_tx = dag.get_with_hash(root_tx_hash).unwrap(); // Get root tx

        assert_eq!(
            found_root_tx.transaction.transaction_data.payload,
            b"test transaction payload"
        ); // Ensure is same transaction

        assert_eq!(dag.write_to_disk(), Ok(())); // Close dag
    }
}
