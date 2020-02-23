use ed25519_dalek; // Import the edwards25519 digital signature library

use super::super::super::common::address::Address;
use super::transaction::Transaction;
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

    /// Verify the signature against a target transaction that has this signature attached to it.
    pub fn verify_tx(&self, transaction: &Transaction) -> bool {
        // Get the public key of the sender of the transaction
        if let Ok(sender_kp) = self.public_key() {
            // Make sure that the sender has the same keypair as that in the tx
            transaction.transaction_data.sender == Address::from_public_key(&sender_kp)
                && self.verify(&*transaction.hash)
        } else {
            // If the sender doesn't have a keypair, the tx can't be valid
            false
        }
    }

    // Gets the signature associated with the signature (deserialize with bincode).
    pub fn signature(&self) -> Result<ed25519_dalek::Signature, failure::Error> {
        Ok(bincode::deserialize(&self.signature_bytes)?)
    }

    /// Gets the public key associated with the signature.
    pub fn public_key(&self) -> Result<ed25519_dalek::PublicKey, failure::Error> {
        Ok(bincode::deserialize(&self.public_key_bytes)?)
    }

    /// Gets the address associated with the signature.
    pub fn address(&self) -> Result<Address, failure::Error> {
        // Get the public key of the signature
        let pub_key = self.public_key()?;

        // Convert the public key to an address
        Ok(Address::from_public_key(&pub_key))
    }
}
