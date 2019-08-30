use libp2p::Multiaddr; // Import the libp2p library

use bincode; // Import the bincode serialization library

use super::{super::core::sys::config, client, message, network}; // Import the config, message modules

/// Download a copy of the configuration file for the corresponding network.
pub fn synchronize_for_network(network: network::Network) -> Option<config::Config> {
    let bootstrap_peers: Vec<Multiaddr> = super::peers::get_network_bootstrap_peers(network); // Get bootstrap peers

    // Check actually has bootstrap peers
    if bootstrap_peers.len() == 0 {
        let header = message::Header::new("config", message::Method::Get, vec![network]); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        // Let's request a serialized network config from the above bootstrap peers
        if let Ok(config_bytes) =
            client::broadcast_message_raw_with_response(message, bootstrap_peers)
        {
            // Deserialize the config
            if let Ok(config) = bincode::deserialize(config_bytes.as_slice()) {
                Some(config) // Return the config
            } else {
                None // Nothing to return since we couldn't deserialize it
            }
        } else {
            None // Nothing to return since we couldn't retrieve a config
        }
    } else {
        None
    }
}
