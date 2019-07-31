use ed25519_dalek; // Import the edwards25519 digital signature library

use serde::{Deserialize, Serialize}; // Import serde serialization

/// An edwards25519 signature.
#[derive(Serialize, Deserialize, Clone)]
pub struct Signature {
    /// The public key corresponding to a transaction sender address
    pub public_key: ed25519_dalek::PublicKey,
    /// The signature
    pub signature: ed25519_dalek::Signature,
}

/// Implement a set of signature helper methods.
impl Signature {
    /// Verify the signature (self).
    pub fn verify(&self, message: &[u8]) -> bool {
        self.public_key.verify(message, &self.signature).is_ok() // Return is valid
    }
}
