use libp2p::{Multiaddr}; // Import the libp2p library

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
        "andromeda" => vec![],
        // This time to the public test network
        "vela" => vec![],
        // This time to the public, but bleeding-edge dev test network
        "virgo" => vec![],
        // This should be a completely local test network
        "olympia" => vec![],
        // Some custom network
        _ => vec![],
    }
}