use libp2p::Multiaddr; // Import the libp2p library

use super::{super::core::sys::config, message, network}; // Import the config, message modules

/// Download a copy of the configuration file for the corresponding network.
pub fn synchronize_for_network(network: network::Network) -> Option<config::Config> {
    let bootstrap_peers: Vec<Multiaddr> = super::peers::get_network_bootstrap_peers(network); // Get bootstrap peers

    // Check actually has bootstrap peers
    if bootstrap_peers.len() == 0 {
        let header = message::Header::new("config", message::Method::Get, vec![network]); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        None // Nothing to synchronize
    } else {
        None
    }
}
