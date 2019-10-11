use libp2p::{Multiaddr, identity::Keypair}; // Import libp2p

use super::super::{
    super::{core::types::graph, crypto::hash},
    client, message, network,
}; // Import library modules

/// Synchronize a local transaction graph against a remote copy.
pub fn synchronize_for_network_against_head(
    dag: &mut graph::Graph,
    network: network::Network,
    peers: Vec<Multiaddr>,
    keypair: Keypair,
) -> Result<(), client::CommunicationError> {
    // Check must have head
    if dag.nodes.len() > 0 {
        // Set head to last node in graph
        if let Ok(some_head) = dag.get(dag.nodes.len() - 1) {
            // Ensure a head exists
            if let Some(raw_head) = some_head {
                let mut head = raw_head.hash; // Get head hash

                let target = synchronize_target_for_network(network, peers.clone(), keypair.clone())?; // Synchronize target node

                // Keep synchronizing until the head is the target
                while head != target {
                    let new_head = synchronize_next_for_network(head, network, peers.clone(), keypair.clone())?; // Synchronize next node

                    dag.push(new_head.transaction, new_head.state_entry); // Add node to dag

                    head = new_head.hash; // Set head
                }

                Ok(()) // Done!
            } else {
                Err(client::CommunicationError::Unknown) // Idk
            }
        } else {
            Err(client::CommunicationError::Unknown) // Idk
        }
    } else {
        let root = synchronize_root_for_network(network, peers.clone(), keypair.clone())?; // Synchronize root node

        *dag = graph::Graph::new(root.transaction, network.to_str()); // Construct a new graph

        synchronize_for_network_against_head(dag, network, peers, keypair) // Return synchronized dag
    }
}

/// Request a copy of the head hash for the network's dag.
fn synchronize_target_for_network(
    network: network::Network,
    peers: Vec<Multiaddr>,
    keypair: Keypair,
) -> Result<hash::Hash, client::CommunicationError> {
    let header = message::Header::new(
        "ledger::transactions",
        message::Method::Last { summarize: false },
        vec![network],
    ); // Initialize message header declaring we want to download the hash of the last node in the graph
    let message = message::Message::new(header, vec![]); // Initialize message

    // Request the last node hash from our target peers
    match client::broadcast_message_raw_with_response(message, peers, keypair) {
        // Yay! We've got the head hash!
        Ok(raw_hash) => Ok(hash::Hash::new(raw_hash)),
        // An error occurred, return it
        Err(e) => Err(e),
    }
}

/// Download a copy of the root node corresponding to the given network.
fn synchronize_root_for_network(
    network: network::Network,
    peers: Vec<Multiaddr>,
    keypair: Keypair,
) -> Result<graph::Node, client::CommunicationError> {
    let header = message::Header::new(
        "ledger::transactions",
        message::Method::First { summarize: false },
        vec![network],
    ); // Initialize a message header declaring we want to download the
    let message = message::Message::new(header, vec![]); // Initialize message

    // Request the first node from our target peers
    match client::broadcast_message_raw_with_response(message, peers, keypair) {
        // All targeted peers responded
        Ok(node_bytes) => {
            if let Ok(node) = bincode::deserialize(node_bytes.as_slice()) {
                Ok(node) // Return the node
            } else {
                Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error
            }
        }
        // A communication error occurred, return it
        Err(e) => Err(e),
    }
}

/// Download a copy of the next node in the graph.
fn synchronize_next_for_network(
    current_node_hash: hash::Hash,
    network: network::Network,
    peers: Vec<Multiaddr>,
    keypair: Keypair,
) -> Result<graph::Node, client::CommunicationError> {
    let header = message::Header::new(
        "ledger::transactions",
        message::Method::Next { summarize: false },
        vec![network],
    ); // Initialize message header declaring we want to download the next target node
    let message = message::Message::new(header, current_node_hash.to_vec()); // Initialize message

    // Request the next node from our target peers
    match client::broadcast_message_raw_with_response(message, peers, keypair) {
        // All targeted peers responded
        Ok(node_bytes) => {
            if let Ok(node) = bincode::deserialize(node_bytes.as_slice()) {
                Ok(node) // Return the node
            } else {
                Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error
            }
        }
        // A communication error occurred, return it
        Err(e) => Err(e),
    }
}
