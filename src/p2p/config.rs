use libp2p::Multiaddr; // Import the libp2p library

use super::super::core::sys::config; // Import the config module

/// Download a copy of the configuration file for the corresponding network.
pub fn synchronize_for_network(network_name: &str) -> Option<config::Config> {
    let bootstrap_peers: Vec<Multiaddr> = super::peers::get_network_bootstrap_peers(network_name); // Get bootstrap peers

    // Check actually has bootstrap peers
    if bootstrap_peers.len() == 0 {
        None // Nothing to synchronize
    } else {
        None
    }
}
