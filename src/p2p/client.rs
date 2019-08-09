use super::super::core::sys::system; // Import the system module

/// A network client.
pub struct Client {
    /// The active SummerCash runtime environment
    pub runtime: system::System,
}