use libp2p::Multiaddr; // Import the libp2p library

use bincode; // Import the bincode serialization library

use super::super::{
    super::{core::sys::config, crypto::blake2},
    client, message, network,
}; // Import the config, message modules

/// Compare a local copy of the configuration file to a remote version. Syncs
/// if necessary.
pub fn synchronize_for_network_against_existing(
    existing_config: config::Config,
    network: network::Network,
    peers: Vec<Multiaddr>,
) -> Result<config::Config, client::CommunicationError> {
    // Check actually has bootstrap peers
    if peers.len() != 0 {
        let header = message::Header::new(
            "config",
            message::Method::Get { summarize: true },
            vec![network],
        ); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        // Let's request a serialized network config from the above bootstrap peers
        if let Ok(config_bytes_hash) =
            client::broadcast_message_raw_with_response(message, peers.clone())
        {
            // Serialize existing configuration
            if let Ok(config_bytes) = bincode::serialize(&existing_config) {
                let hash = blake2::hash_slice(config_bytes.as_slice()); // Hash config bytes

                // Check local config is up to date
                if hash.to_vec() == config_bytes_hash {
                    Ok(existing_config) // Return local config
                } else {
                    synchronize_for_network(network, peers) // Sync
                }
            } else {
                Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error (though this isn't necessarily related to communications)
            }
        } else {
            Err(client::CommunicationError::MajorityDidNotRespond) // Nothing to return since we couldn't retrieve a config
        }
    } else {
        Err(client::CommunicationError::NoAvailablePeers) // Return an error since there are no available peers
    }
}

/// Download a copy of the configuration file for the corresponding network.
pub fn synchronize_for_network(
    network: network::Network,
    peers: Vec<Multiaddr>,
) -> Result<config::Config, client::CommunicationError> {
    // Check actually has bootstrap peers
    if peers.len() != 0 {
        let header = message::Header::new(
            "config",
            message::Method::Get { summarize: false },
            vec![network],
        ); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        // Let's request a serialized network config from the above bootstrap peers
        if let Ok(config_bytes) = client::broadcast_message_raw_with_response(message, peers) {
            // Deserialize the config
            if let Ok(config) = bincode::deserialize(config_bytes.as_slice()) {
                Ok(config) // Return the config
            } else {
                Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error (though this isn't necessarily related to communications)
            }
        } else {
            Err(client::CommunicationError::MajorityDidNotRespond) // Nothing to return since we couldn't retrieve a config
        }
    } else {
        Err(client::CommunicationError::NoAvailablePeers) // Return an error since there are no available peers
    }
}
