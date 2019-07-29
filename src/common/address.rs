extern crate hex; // Link hex encoding library

use std::ops::{Deref, DerefMut}; // Allow implementation of deref&defermut

/// The length of a standard address (32 bytes).
pub const ADDRESS_SIZE: usize = 32;

/// A standard 32-byte blake2 hash of an account's public key.
pub struct Address([u8; ADDRESS_SIZE]);

/* BEGIN ADDRESS TYPE METHODS */

/// Implement the std deref op.
impl Deref for Address {
    type Target = [u8; ADDRESS_SIZE]; // Initialize target

    // Implement deref
    fn deref(&self) -> &Self::Target {
        &self.0 // Return self
    }
}

/// Implement the std deref_mut op.
impl DerefMut for Address {
    // Implement deref_mut
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 // Return mut self
    }
}

// Implement the std as_ref op.
impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0 // Lmao
    }
}

/// Implement a set of address helper methods.
impl Address {
    /// Convert an address to a hex-encoded string.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::common::address; // Import the address utility
    ///
    /// let address = address::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]); // [0, 1...] (values after index of 32 trimmed)
    ///
    /// let hex_encoded = address.to_str(); //
    /// ```
    pub fn to_str(&self) -> String {
        return hex::encode(self); // Return string val
    }
}

/* END ADDRESS TYPE METHODS */

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
    let mut buffer: Address = Address([0; ADDRESS_SIZE]); // Initialize address buffer

    let mut modifiable_b: Vec<u8> = b; // Get local scope b value
    modifiable_b.truncate(ADDRESS_SIZE); // Trim past index 32

    buffer.copy_from_slice(modifiable_b.as_slice()); // Copy contents of vec into buffer

    return buffer; // Return contents of buffer
}

/// Convert a given hex-encoded string to an address instance.Address
///
/// # Example
///
/// ```
/// use summercash::common::address; // Import the address utility
///
/// let address = address::from_str("hex_encoded_address"); // TODO: Put an actual address here for doc completion
pub fn from_str(s: &str) -> Result<Address, hex::FromHexError> {
    let b = hex::decode(s); // Decode hex address value

    match b {
        Ok(bytes) => return Ok(new(bytes)), // Return address value
        Err(error) => {
            return Err(error); // Return result containing error
        }
    }; // Handle errors
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import names from outside module

    #[test]
    fn test_to_str() {
        let address = new(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]); // [0, 1...] (values after index of 32 trimmed)

        println!("{}", address.to_str());
    }
}
