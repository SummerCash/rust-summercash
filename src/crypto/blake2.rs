extern crate blake2; // Link blake2 hashing library
extern crate hex; // Link hex encoding library

use blake2::{Blake2s, Digest}; // Use blake2s

use super::hash; // Import the hash type module

/* BEGIN EXPORTED METHODS */

/// Hash a given slice input, b via blake2s.
/// 
/// # Example
/// 
/// ```
/// use summercash::crypto::blake2; // Import the blake2 hashing utility
/// 
/// let hash = blake2::hash_slice(&[1, 2, 3, 4]);
/// ```
pub fn hash_slice(b: &[u8]) -> hash::Hash {
    let mut hasher = Blake2s::new(); // Init hasher
    
    hasher.input(b); // Set input

    return hash::new(hasher.result()[..].to_vec()); // Hash input
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import names from outside module

    #[test]
    fn test_hash_slice() {
        let hashed = hash_slice(b"hello world"); // Hash a test message

        assert_eq!(hashed.to_str(), "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"); // Ensure properly hashed
    }
}