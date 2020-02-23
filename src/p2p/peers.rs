use super::network; // Import the network module

use libp2p::{Multiaddr, PeerId}; // Import the libp2p library

/// Get a list of bootstrap peers for a particular network.
/// TODO: Hard-code peer ids
pub fn get_network_bootstrap_peers(network: network::Network) -> Vec<(PeerId, Multiaddr)> {
    match network {
        // We're trying to sync to the main SummerCash network
        network::Network::MainNetwork => vec![(
            get_peer_id("QmQZJ5p27AcQk6QHPB3PuxyT6hn8RB488j67NGhcJ84Qmv"),
            get_multiaddr("/dns4/node1.summer.cash/tcp/2048"),
        )],
        // This time to the public test network
        network::Network::PublicTestNetwork => vec![(
            get_peer_id("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),
            get_multiaddr("/ip4/108.41.124.60/tcp/4096"),
        )],
        // This time to the public, but bleeding-edge dev test network
        network::Network::DevTestNetwork => vec![(
            get_peer_id("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),
            get_multiaddr("/ip4/108.41.124.60/tcp/8192"),
        )],
        // This should be a completely local test network
        network::Network::LocalTestNetwork => vec![],
    }
}

/// Get a list of bootstrap peer addresses for a particular network.
pub fn get_network_bootstrap_peer_addresses(network: network::Network) -> Vec<Multiaddr> {
    match network {
        // We're trying to sync to the main SummerCash network
        network::Network::MainNetwork => vec![get_multiaddr("/ip4/108.41.124.60/tcp/2048")],
        // This time to the public test network
        network::Network::PublicTestNetwork => vec![get_multiaddr("/ip4/108.41.124.60/tcp/4096")],
        // This time to the public, but bleeding-edge dev test network
        network::Network::DevTestNetwork => vec![get_multiaddr("/ip4/108.41.124.60/tcp/8192")],
        // This should be a completely local test network
        network::Network::LocalTestNetwork => vec![],
    }
}

/* BEGIN INTERNAL METHODS */

fn get_multiaddr(addr_str: &str) -> Multiaddr {
    // Parse multiaddr
    if let Ok(addr) = addr_str.parse() {
        addr // Return address
    } else {
        Multiaddr::empty() // Nil multiaddr
    }
}

fn get_peer_id(peer_id: &str) -> PeerId {
    // Parse multiaddr
    if let Ok(peer_id) = peer_id.parse() {
        peer_id // Return peer id
    } else {
        PeerId::random() // Random peer ID
    }
}

/* END INTERNAL METHODS */

#[cfg(test)]
mod tests {
    use super::super::network;
    use super::*;

    #[test]
    fn test_get_network_bootstrap_nodes() {
        let main_network_boot_nodes: Vec<Multiaddr> =
            get_network_bootstrap_peer_addresses(network::Network::MainNetwork); // Get main network peers
        assert_eq!(
            *main_network_boot_nodes.get(0).unwrap(),
            get_multiaddr("/ip4/108.41.124.60/tcp/2048")
        ); // Should have one bootstrap node
    }
}
