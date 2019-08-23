use super::super::core::sys::config; // Import the node config module

/// A SummerCash network.
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
            Network::MainNetwork => "andromeda",
            Network::PublicTestNetwork => "vela",
            Network::DevTestNetwork => "virgo",
            Network::LocalTestNetwork => "olympia",
        } // Handle different networks
    }

    /// Derive a p2p protocol path from a given message protocol.
    pub fn derive_p2p_protocol_path(&self, protocol: &str) -> String {
        format!(
            "/smc/{}/{}/{}",
            self.to_str(),
            config::NODE_VERSION,
            protocol
        ) // Return formatted
    }
}

/// The main SummerCash network name.
pub static MAIN_NETWORK_NAME: &str = "andromeda";

/// The stable public SummerCash test network name.
pub static PUBLIC_TEST_NETWORK_NAME: &str = "vela";

/// The bleeding-edge SummerCash network name.
pub static DEV_TEST_NETWORK_NAME: &str = "virgo";

/// The local SummerCash test network name.
pub static LOCAL_TEST_NETWORK_NAME: &str = "olympia";
