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
    fn to_str(self) -> &'static str {
        match self {
            Network::MainNetwork => MAIN_NETWORK_NAME,
            Network::PublicTestNetwork => PUBLIC_TEST_NETWORK_NAME,
            Network::DevTestNetwork => DEV_TEST_NETWORK_NAME,
            Network::LocalTestNetwork => LOCAL_TEST_NETWORK_NAME,
        } // Handle different networks
    }
}

/// Implement conversions from a network primitive to a referenced string.
impl Into<&str> for Network {
    /// Converts the network primitive identiofier to a referenced string.
    fn into(self) -> &'static str {
        // Return the string version of the network identifier
        self.to_str()
    }
}

/// Implement conversions from a network primitive to an owned string.
impl Into<String> for Network {
    /// Converts the network primitive identifier into an owned string.
    fn into(self) -> String {
        // Convert the network to an owned string through the existing to_str impl
        self.to_str().to_owned()
    }
}

impl From<String> for Network {
    /// Converts the given string to a network.
    fn from(s: String) -> Self {
        // Convert the string to a network
        Self::from(s.as_ref())
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
