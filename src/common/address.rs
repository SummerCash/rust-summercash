extern crate ed25519_dalek; // Link edwards25519 library
extern crate hex; // Link hex encoding library

use ed25519_dalek::{Keypair, PublicKey}; // Import the edwards25519 digital signature library

use super::super::crypto::blake2;
use super::super::crypto::hash; // Import the hash library // Import the blake2 hashing library

/// The length of a standard address (32 bytes).
pub const ADDRESS_SIZE: usize = 32;

/// A standard 32-byte blake2 hash of an account's public key.
pub type Address = hash::Hash;

/* BEGIN EXPORTED METHODS */

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

/// Derive an address from a given edwards25519 public key.
///
/// # Example
///
/// ```
/// extern crate ed25519_dalek;
/// extern crate rand;
///
/// use rand::rngs::OsRng; // Import the os's rng
/// use ed25519_dalek::Keypair; // Import the ed25519 keypair+signature types
///
/// use summercash::common::address; // Import the address utility
///
/// let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
/// let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair
///
/// let address = address::from_public_key(&keypair.public); // Derive address
/// ```
pub fn from_public_key(public_key: &PublicKey) -> Address {
    return blake2::hash_slice(&public_key.to_bytes()); // Hash public key
}

/// Derive an address from a given edwards25519 keypair.
///
/// # Example
///
/// ```
/// extern crate ed25519_dalek;
/// extern crate rand;
///
/// use rand::rngs::OsRng; // Import the os's rng
/// use ed25519_dalek::Keypair; // Import the ed25519 keypair type
///
/// use summercash::common::address; // Import the address utility
///
/// let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
/// let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair
///
/// let address = address::from_key_pair(&keypair); // Derive address
/// ```
pub fn from_key_pair(key_pair: &Keypair) -> Address {
    return blake2::hash_slice(&key_pair.public.to_bytes()); // Hash public key
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    extern crate rand; // Link random library

    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng; // Import the os's rng

    use super::*; // Import names from our parent module

    #[test]
    fn test_new() {
        let address = new(hex::decode(
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b",
        )
        .unwrap()); // Construct an address from a pre-determined hex value

        assert_eq!(
            address.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure address was constructed properly, and that to_str() works
    }

    #[test]
    fn test_from_str() {
        let address =
            from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b").unwrap(); // Convert a known safe address hex value to an address instance

        assert_eq!(
            address.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure our original input was preserved
    }

    #[test]
    fn test_from_public_key() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
        let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair

        let address = from_public_key(&keypair.public); // Derive address from public key

        assert_eq!(
            address.to_str(),
            blake2::hash_slice(&keypair.public.to_bytes()).to_str()
        ); // Ensure address properly derived
    }

    #[test]
    fn test_from_key_pair() {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness
        let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair

        let address = from_key_pair(&keypair); // Derive address from pair

        assert_eq!(
            address.to_str(),
            blake2::hash_slice(&keypair.public.to_bytes()).to_str()
        ); // Ensure address properly derived
    }
}
