use libp2p::Multiaddr; // Import the libp2p library

/// The main SummerCash network name.
pub static MAIN_NETWORK_NAME: &str = "andromeda";

/// The stable public SummerCash test network name.
pub static PUBLIC_TEST_NETWORK_NAME: &str = "vela";

/// The bleeding-edge SummerCash network name.
pub static DEV_TEST_NETWORK_NAME: &str = "virgo";

/// The local SummerCash test network name.
pub static LOCAL_TEST_NETWORK_NAME: &str = "olympia";

/// Get a list of bootstrap peers for a particular network.
pub fn get_network_bootstrap_peers(network_name: &str) -> Vec<Multiaddr> {
    match network_name {
        // We're trying to sync to the main SummerCash network
        "andromeda" => vec![get_multiaddr("/ip4/108.41.124.60/tcp/2048")],
        // This time to the public test network
        "vela" => vec![get_multiaddr("/ip4/108.41.124.60/tcp/4096")],
        // This time to the public, but bleeding-edge dev test network
        "virgo" => vec![get_multiaddr("/ip4/108.41.124.60/tcp/8192")],
        // This should be a completely local test network
        "olympia" => vec![],
        // Some custom network
        _ => vec![],
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

/* END INTERNAL METHODS */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_network_bootstrap_nodes() {
        let main_network_boot_nodes: Vec<Multiaddr> =
            get_network_bootstrap_peers(MAIN_NETWORK_NAME); // Get main network peers
        assert_eq!(
            *main_network_boot_nodes.get(0).unwrap(),
            get_multiaddr("/ip4/108.41.124.60/tcp/2048")
        ); // Should have one bootstrap node
    }
}
