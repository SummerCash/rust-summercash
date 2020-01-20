use ed25519_dalek::{Keypair, PublicKey}; // Import the edwards25519 digital signature library

use super::super::crypto::blake3;
use super::super::crypto::hash; // Import the hash library // Import the blake3 hashing library

/// The length of a standard address (32 bytes).
pub const ADDRESS_SIZE: usize = 32;

/// A standard 32-byte blake3 hash of an account's public key.
pub type Address = hash::Hash;

/* BEGIN EXPORTED METHODS */

impl Address {
    /// Get a zero-value instance of an address.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::common::address; // Import the address utility
    ///
    /// let default_address = address::Address::default(); // Get default address
    pub fn default() -> Address {
        Address::new(vec![0; 32]) // Return zero value
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
    /// let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair
    ///
    /// let address = address::Address::from_public_key(&keypair.public); // Derive address
    /// ```
    pub fn from_public_key(public_key: &PublicKey) -> Address {
        blake3::hash_slice(&public_key.to_bytes()) // Hash public key
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
    /// let mut csprng = OsRng{}; // Generate source of randomness
    /// let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair
    ///
    /// let address = address::Address::from_key_pair(&keypair); // Derive address
    /// ```
    pub fn from_key_pair(key_pair: &Keypair) -> Address {
        blake3::hash_slice(&key_pair.public.to_bytes()) // Return hashed public key
    }
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
    fn test_default() {
        let default_address = Address::default(); // Get default address val

        assert_eq!(
            default_address.to_str(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        ); // Ensure default address is all zeros
    }

    #[test]
    fn test_new() {
        let address = Address::new(
            hex::decode("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b")
                .unwrap(),
        ); // Construct an address from a pre-determined hex value

        assert_eq!(
            address.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure address was constructed properly, and that to_str() works
    }

    #[test]
    fn test_from_str() {
        let address =
            Address::from_str("9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b")
                .unwrap(); // Convert a known safe address hex value to an address instance

        assert_eq!(
            address.to_str(),
            "9aec6806794561107e594b1f6a8a6b0c92a0cba9acf5e5e93cca06f781813b0b"
        ); // Ensure our original input was preserved
    }

    #[test]
    fn test_from_public_key() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair

        let address = Address::from_public_key(&keypair.public); // Derive address from public key

        assert_eq!(
            address.to_str(),
            blake3::hash_slice(&keypair.public.to_bytes()).to_str()
        ); // Ensure address properly derived
    }

    #[test]
    fn test_from_key_pair() {
        let mut csprng = OsRng {}; // Generate source of randomness
        let keypair: Keypair = Keypair::generate(&mut csprng); // Generate key pair

        let address = Address::from_key_pair(&keypair); // Derive address from pair

        assert_eq!(
            address.to_str(),
            blake3::hash_slice(&keypair.public.to_bytes()).to_str()
        ); // Ensure address properly derived
    }
}
