use serde::{Deserialize, Serialize}; // Import serde serialization

/// The main SummerCash network name.
pub static MAIN_NETWORK_NAME: &str = "andromeda";

/// The stable public SummerCash test network name.
pub static PUBLIC_TEST_NETWORK_NAME: &str = "vela";

/// The bleeding-edge SummerCash network name.
pub static DEV_TEST_NETWORK_NAME: &str = "virgo";

/// The local SummerCash test network name.
pub static LOCAL_TEST_NETWORK_NAME: &str = "olympia";

/// A SummerCash network.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Network {
    /// The main SummerCash network.
    MainNetwork,
    /// The stable public SummerCash test network.
    PublicTestNetwork,
    /// The bleeding-edge SummerCash network name.
    DevTestNetwork,
    /// The local SummerCash test network name.
    LocalTestNetwork,
}

/// Implement a set of network enum helper methods.
impl Network {
    /// Get the string representation of a particular network.
    pub fn to_str(&self) -> &str {
        match *self {
            Network::MainNetwork => MAIN_NETWORK_NAME,
            Network::PublicTestNetwork => PUBLIC_TEST_NETWORK_NAME,
            Network::DevTestNetwork => DEV_TEST_NETWORK_NAME,
            Network::LocalTestNetwork => LOCAL_TEST_NETWORK_NAME,
        } // Handle different networks
    }
}

/// Implement string to network conversion.
impl From<&str> for Network {
    /// Convert a given string to a network.
    fn from(s: &str) -> Network {
        // Handle different network names
        match s {
            // The main net
            "andromeda" => Network::MainNetwork,
            // The public test net
            "vela" => Network::PublicTestNetwork,
            // The dev test net
            "virgo" => Network::DevTestNetwork,
            // A local test net
            "olympia" => Network::LocalTestNetwork,
            // Let's assume this is a local test net, since it doesn't have a proper name
            _ => Network::LocalTestNetwork,
        }
    }
}
