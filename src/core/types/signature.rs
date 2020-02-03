use ed25519_dalek; // Import the edwards25519 digital signature library

use serde::{Deserialize, Serialize}; // Import serde serialization

/// An edwards25519 signature.
#[derive(Serialize, Deserialize, Clone)]
pub struct Signature {
    /// The public key corresponding to a transaction sender address
    pub(crate) public_key_bytes: Vec<u8>,
    /// The signature
    pub(crate) signature_bytes: Vec<u8>,
}

/// Implement a set of signature helper methods.
impl Signature {
    /// Verify the signature (self).
    pub fn verify(&self, message: &[u8]) -> bool {
        // Get the signature's public key
        let pub_key = if let Ok(pk) = self.public_key() {
            pk
        } else {
            return false;
        };

        // Get the signature's signature
        let sig = if let Ok(s) = self.signature() {
            s
        } else {
            return false;
        };

        pub_key.verify(message, &sig).is_ok() // Return is valid
    }

    // Gets the signature associated with the signature (deserialize with bincode).
    pub fn signature(&self) -> Result<ed25519_dalek::Signature, failure::Error> {
        Ok(bincode::deserialize(&self.signature_bytes)?)
    }

    /// Gets the public key associated with the signature.
    pub fn public_key(&self) -> Result<ed25519_dalek::PublicKey, failure::Error> {
        Ok(bincode::deserialize(&self.public_key_bytes)?)
    }
}
