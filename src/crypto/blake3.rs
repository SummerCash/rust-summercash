use blake3::Hasher; // Use blake3

use super::hash; // Import the hash type module

/* BEGIN EXPORTED METHODS */

/// Hash a given slice input, b via blake3b.
///
/// # Example
///
/// ```
/// use summercash::crypto::blake3; // Import the blake3 hashing utility
///
/// let hash = blake3::hash_slice(&[1, 2, 3, 4]);
/// ```
pub fn hash_slice(b: &[u8]) -> hash::Hash {
    let mut hasher = Hasher::new(); // Init hasher
    hasher.update(b); // Put the slice that the user gave us into the hasher

    hash::Hash::new(hasher.finalize().as_bytes()[..].to_vec()) // Hash and return it!
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
            "FVPfbg9bK7mj7jnaSRXhuVcVakkXcjMPgSwxmauUofYf"
        ); // Ensure properly hashed
    }
}
