use super::super::{super::core::types::graph, network, client}; // Import the hash module

/// Synchronize a local transaction graph against a remote copy.
pub fn synchronize_for_network_against_head(graph: &graph::Graph, network: network::Network, peers: Vec<Multiaddr>) -> Result<client::CommunicationError, &graph::Graph> {
    // Get graph head
    if let Some(head) = graph.nodes.get(graph.nodes.len()-1) {
        
    } else {

    }
}