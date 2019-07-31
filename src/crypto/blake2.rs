use blake2::{Blake2b, Digest}; // Use blake2b

use super::hash; // Import the hash type module

/* BEGIN EXPORTED METHODS */

/// Hash a given slice input, b via blake2b.
///
/// # Example
///
/// ```
/// use summercash::crypto::blake2; // Import the blake2 hashing utility
///
/// let hash = blake2::hash_slice(&[1, 2, 3, 4]);
/// ```
pub fn hash_slice(b: &[u8]) -> hash::Hash {
    let mut hasher = Blake2b::new(); // Init hasher

    hasher.input(b); // Set input

    hash::Hash::new(hasher.result()[..].to_vec()) // Hash input
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import names from outside module

    #[test]
    fn test_hash_slice() {
        let hashed = hash_slice(b"hello world"); // Hash a test message

        assert_eq!(
            hashed.to_str(),
            "021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbc"
        ); // Ensure properly hashed
    }
}
