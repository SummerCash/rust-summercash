extern crate hex; // Link hex encoding library

use super::super::crypto::hash; // Import the hash library

/// The length of a standard address (32 bytes).
pub const ADDRESS_SIZE: usize = 32;

/// A standard 32-byte blake2 hash of an account's public key.
pub type Address = hash::Hash;

/// Initialize a new address instance from a given byte vector.
///
/// # Example
///
/// ```
/// use summercash::common::address; // Import the address utility
///
/// let address = address::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]); // [0, 1...] (values after index of 32 trimmed)
/// ```
pub fn new(b: Vec<u8>) -> Address {
    return hash::new(b); // Return initialized address
}

/// Convert a given hex-encoded address string to an address instance.
///
/// # Example
///
/// ```
/// use summercash::common::address; // Import the address utility
///
/// let address = address::from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"); // Some address instance
/// ```
pub fn from_str(s: &str) -> Result<Address, hex::FromHexError> {
    return hash::from_str(s); // Return result
}
