use libp2p::Multiaddr; // Import libp2p

use super::super::{super::{core::types::graph, crypto::hash}, client, network, message}; // Import library modules

/// Synchronize a local transaction graph against a remote copy.
pub fn synchronize_for_network_against_head(
    dag: &graph::Graph,
    network: network::Network,
    peers: Vec<Multiaddr>,
) -> Result<(), client::CommunicationError> {
    // Check must have head
    if dag.nodes.len() > 0 {
        // Set head to last node in graph
        if let Ok(some_head) = dag.get(dag.nodes.len()-1) {
            // Ensure a head exists
            if let Some(head) = some_head {
                let target = synchronize_target_for_network(network, peers)?; // Synchronize target node

                // Keep synchronizing until the head is the target
                while head.hash != target {
                    
                }
            } else {
                client::CommunicationError::Unknown // Idk
            }
        } else {
            client::CommunicationError::Unknown // Idk
        }
    } else {
        // Synchronize root node
        match synchronize_root_for_network(network, peers) {
            // A copy of the root node was successfully downloaded
            Ok(root_node) => {
                *dag = graph::Graph::new(root_node.transaction, network.to_str()); // Construct a new graph

                synchronize_for_network_against_head(dag, network, peers) // Return synchronized dag
            },
            // An error was encountered while synchronizing the root node
            Err(e) => e,
        }
    }
}

/// Request a copy of the head hash for the network's dag.
fn synchronize_target_for_network(network: network::Network, peers: Vec<Multiaddr>) -> Result<hash::Hash, client::CommunicationError> {
    let header = message::Header::new("ledger::transactions", message::Method::Last{summarize: false}, vec![network]); // Initialize message header declaring we want to download the hash of the last node in the graph
    let message = message::Message::new(header, vec![]); // Initialize message

    // Request the last node hash from our target peers
    match client::broadcast_message_raw_with_response(message, peers) {
        // Yay! We've got the head hash!
        Ok(raw_hash) => Ok(hash::Hash::new(raw_hash)),
        // An error occurred, return it
        e => e,
    }
}

/// Download a copy of the root node corresponding to the given network.
fn synchronize_root_for_network(network: network::Network, peers: Vec<Multiaddr>) -> Result<graph::Node, client::CommunicationError> {
    let header = message::Header::new("ledger::transactions", message::Method::First{summarize: false}, vec![network]); // Initialize a message header declaring we want to download the 
    let message = message::Message::new(header, vec![]); // Initialize message

    // Request the first node from our targe tpeers
    match client::broadcast_message_raw_with_response(message, peers) {
        // All targeted peers responded
        Ok(node_bytes) => if let Ok(node) = bincode::deserialize(node_bytes.as_slice()) {
            Ok(node) // Return the node
        } else {
            Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error
        },
        // A communication error occurred, return it
        e => e,
    }
}

/// Download a copy of the next node in the graph.
fn synchronize_next_for_network(current_node_hash: hash::Hash, network: network::Network, peers: Vec<Multiaddr>) -> Result<graph::Node, client::CommunicationError> {
    let header = message::Header::new("ledger::transactions", message::Method::Next{summarize: false}, vec![network]); // Initialize message header declaring we want to download the next target node
    let message = message::Message::new(header, current_node_hash.to_vec()); // Initialize message

    // Request the next node from our target peers
    match client::broadcast_message_raw_with_response(message, peers) {
        // All targeted peers responded
        Ok(node_bytes) => if let Ok(node) = bincode::deserialize(node_bytes.as_slice()) {
            Ok(node) // Return the node
        } else {
            Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error
        },
        // A communication error occurred, return it
        e => e,
    }
}