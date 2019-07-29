extern crate blake2; // Link blake2 hashing library

use blake2::{Blake2s, Digest}; // Use blake2s

/// Hash a given slice input, b via blake2s.
/// 
/// # Example
/// 
/// ```
/// use summercash::crypto::blake2; // Import the blake2 hashing utility
/// 
/// let hash = blake2::hash_slice()
/// ```
pub fn hash_slice(b: &[u8]) -> &[u8] {

}
pub fn blake2(b: &[u8]) -> &[u8] {

}