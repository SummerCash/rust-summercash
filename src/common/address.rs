extern crate hex; // Link hex encoding library

use super::super::crypto::hash; // Import the hash library

/// The length of a standard address (32 bytes).
pub const ADDRESS_SIZE: usize = 32;

/// A standard 32-byte blake2 hash of an account's public key.
pub type Address = hash::Hash;