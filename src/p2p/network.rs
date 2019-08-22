use libp2p::core::ProtocolName; // Import the libp2p protocol name protocol

use super::super::core::sys::config; // Import the node config module

/// The main SummerCash network name.
pub static MAIN_NETWORK_NAME: &str = "andromeda";

/// The stable public SummerCash test network name.
pub static PUBLIC_TEST_NETWORK_NAME: &str = "vela";

/// The bleeding-edge SummerCash network name.
pub static DEV_TEST_NETWORK_NAME: &str = "virgo";

/// The local SummerCash test network name.
pub static LOCAL_TEST_NETWORK_NAME: &str = "olympia";

/// The SummerCash protocol.
pub enum SummerCash {
    Andromeda,
    Vela,
    Virgo,
    Olympia,
}

/// Implement the ProtocolName protocol type.
impl ProtocolName for SummerCash {
    fn protocol_name(&self) -> &[u8] {
        match *self {
            SummerCash::Andromeda => format!("/smc/andromeda/{}", config::NODE_VERSION.as_bytes()),
            SummerCash::Vela => format!("/smc/vela/{}", config::NODE_VERSION.as_bytes()),
            SummerCash::Virgo => format!("/smc/virgo/{}", config::NODE_VERSION.as_bytes()),
            SummerCash::Olympia => format!("/smc/virgo/{}", config::NODE_VERSION.as_bytes()),
        }
    }
}
