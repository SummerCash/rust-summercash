use serde::{Deserialize, Serialize}; // Import serde serialization

use std::{
    fmt,
    ops::{Deref, DerefMut},
}; // Allow implementation of deref&defer_mut

// The length of a standard hash (32 bytes).
pub const HASH_SIZE: usize = 32;

// A standard 32-byte blake3 hash.
#[derive(Serialize, Deserialize, Clone, Copy, Eq, Hash, Debug)]
pub struct Hash([u8; HASH_SIZE]);

/* BEGIN HASH TYPE METHODS */

/// Implement the hash partial equality checker.
impl PartialEq for Hash {
    /// Check if self is equivalent to a given hash, other.
    fn eq(&self, other: &Hash) -> bool {
        self.iter().zip(other.iter()).all(|(a, b)| a == b) // Check each byte equal
    }
}

/// Implement the std deref op.
impl Deref for Hash {
    type Target = [u8; HASH_SIZE]; // Initialize target

    // Implement deref
    fn deref(&self) -> &Self::Target {
        &self.0 // Return self
    }
}

/// Implement the std deref_mut op.
impl DerefMut for Hash {
    // Implement deref_mut
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 // Return mut self
    }
}

// Implement the std as_ref op.
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0 // Lmao
    }
}

/// Implement conversion from a hash to a hex-encoded string.
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert the slice to a hex string
        write!(f, "{}", hex::encode(self))
    }
}

impl Default for Hash {
    /// Initializes an empty hash.
    fn default() -> Self {
        // Just make an empty vec
        Self::new(vec![0; 32])
    }
}

/* BEGIN EXPORTED METHODS */

/// Implement a set of hash helper methods.
impl Hash {
    /// Initialize a new hash instance from a given byte vector.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::crypto::hash; // Import the hash utility
    ///
    /// let hash = hash::Hash::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]); // [0, 1...] (values after index of 32 trimmed)
    /// ```
    pub fn new(b: Vec<u8>) -> Hash {
        let mut buffer: Hash = Hash([0; HASH_SIZE]); // Initialize hash buffer

        let mut modifiable_b: Vec<u8> = b; // Get local scope b value
        modifiable_b.truncate(HASH_SIZE); // Trim past index 32

        buffer.copy_from_slice(modifiable_b.as_slice()); // Copy contents of vec into buffer

        buffer // Return contents of buffer
    }

    /// Convert a given hex-encoded hash string to an hash instance.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::crypto::hash; // Import the hash utility
    ///
    /// let hash = hash::Hash::from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"); // Some hash instance
    pub fn from_str(s: &str) -> Result<Hash, hex::FromHexError> {
        let b = hex::decode(s); // Decode hex hash value

        match b {
            Ok(bytes) => return Ok(Hash::new(bytes)), // Return hash value
            Err(error) => return Err(error),          // Return result containing error
        }; // Handle errors
    }

    /// Convert a hash to a hex-encoded string.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::crypto::blake3; // Import the blake3 hashing utility
    ///
    /// let hash = blake3::hash_slice(b"hello world"); // Some hash vector
    ///
    /// let hex_encoded = hash.to_str(); // 9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b
    /// ```
    pub fn to_str(&self) -> String {
        hex::encode(self) // Return string val
    }

    /// Initialize and return a new hash instance derived from self.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::crypto::blake3; // Import the blake3 hashing utility
    ///
    /// let hash = blake3::hash_slice(b"hello world!"); // Some hash vector
    ///
    /// let hash2 = hash.clone(); // Copy value from first hash instance
    /// ```
    pub fn clone(&self) -> Hash {
        let mut hash = Hash::new(vec![0; HASH_SIZE]); // Initialize hash buffer

        hash.copy_from_slice(&**self); // Copy into buffer

        return hash; // Return cloned hash
    }
}

impl From<blake3::Hash> for Hash {
    /// Initializes a new hash from the given blake3 hash.
    fn from(hash: blake3::Hash) -> Self {
        // Return the hash
        Self(*hash.as_bytes())
    }
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import names from outside module

    #[test]
    fn test_to_str() {
        let hash = Hash::new(
            hex::decode("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b")
                .unwrap(),
        ); // Construct a hash from a pre-determined hex value

        assert_eq!(
            hash.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure properly constructed, and that to_string() is equivalent to our original input
    }

    #[test]
    fn test_new() {
        Hash::new(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]); // Construct a hash from pre-determined byte values
        Hash::new(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32,
        ]); // Construct a hash from an overflowing set of byte values
    } // This test simply checks for panics

    #[test]
    fn test_from_str() {
        let hash =
            Hash::from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b")
                .unwrap(); // Convert a known safe hash hex encoding to a hash instance

        assert_eq!(
            hash.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure our original input was preserved
    }
}
