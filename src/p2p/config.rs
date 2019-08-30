use libp2p::Multiaddr; // Import the libp2p library

use bincode; // Import the bincode serialization library

use super::{
    super::{core::sys::config, crypto::blake2},
    client, message, network,
}; // Import the config, message modules

/// Compare a local copy of the configuration file to a remote version. Syncs
/// if necessary.
pub fn synchronize_for_network_against_existing(
    existing_config: config::Config,
    network: network::Network,
) -> Option<config::Config> {
    let bootstrap_peers: Vec<Multiaddr> = super::peers::get_network_bootstrap_peers(network); // Get bootstrap peers

    // Check actually has bootstrap peers
    if bootstrap_peers.len() != 0 {
        let header = message::Header::new("config", message::Method::Summarize, vec![network]); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        // Let's request a serialized network config from the above bootstrap peers
        if let Ok(config_bytes_hash) =
            client::broadcast_message_raw_with_response(message, bootstrap_peers)
        {
            // Serialize existing configuration
            if let Ok(config_bytes) = bincode::serialize(&existing_config) {
                let hash = blake2::hash_slice(config_bytes.as_slice()); // Hash config bytes

                // Check local config is up to date
                if hash.to_vec() == config_bytes_hash {
                    Some(existing_config) // Return local config
                } else {
                    synchronize_for_network(network) // Sync
                }
            } else {
                Some(existing_config) // Return the local config
            }
        } else {
            Some(existing_config) // Nothing to return since we couldn't retrieve a config
        }
    } else {
        Some(existing_config) // Return the local config since we can't sync it anyway
    }
}

/// Download a copy of the configuration file for the corresponding network.
pub fn synchronize_for_network(network: network::Network) -> Option<config::Config> {
    let bootstrap_peers: Vec<Multiaddr> = super::peers::get_network_bootstrap_peers(network); // Get bootstrap peers

    // Check actually has bootstrap peers
    if bootstrap_peers.len() != 0 {
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
