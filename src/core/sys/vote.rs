use ed25519_dalek; // Import the edwards25519 digital signature library

use bincode; // Import serde bincode
use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::{
    common::address::Address,
    crypto::{
        blake3,
        hash::{self, Hash as HashPrim},
    },
}; // Import the hash primitive
use super::super::types::signature; // Import the signature primitive

use std::fmt;

/// A binary, signed vote regarding a particular proposal.
#[derive(Serialize, Deserialize, Clone)]
pub struct Vote {
    /// The hash of the target proposal
    pub target_proposal: hash::Hash,
    /// Whether the voter is in favor of the particular proposal or not
    pub in_favor: bool,
    /// The signature of the voter
    pub signature: Option<signature::Signature>,
}

/// Implement a set of voting helper methods.
impl Vote {
    /// Initialize and sign a new vote instance.
    pub fn new(
        proposal_id: hash::Hash,
        in_favor: bool,
        signature_keypair: ed25519_dalek::Keypair,
    ) -> Vote {
        let mut vote: Vote = Vote {
            target_proposal: proposal_id, // Set proposal ID
            in_favor,                     // Set in favor of proposal
            signature: None,              // No signature yet
        }; // Initialize vote

        // Serialize vote
        vote.signature = Some(signature::Signature {
            public_key_bytes: bincode::serialize(&signature_keypair.public).unwrap_or_default(),
            signature_bytes: bincode::serialize(&signature_keypair.sign(&*vote.hash()))
                .unwrap_or_default(),
        }); // Set signature

        vote // Return initialized vote
    }

    /// Ensures that the signature associated with the vote is authentic.
    ///
    /// # Example
    ///
    /// ```
    /// use summercash::{core::sys::vote::{self, Vote}, crypto::blake3, accounts::account::Account};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let acc: Account = Account::new();
    /// let v: Vote = Vote::new(blake3::hash_slice(b"test"), true, acc.keypair()?);
    ///
    /// assert_eq!(v.valid(), true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn valid(&self) -> bool {
        // Ensure that the vote has a signature attached to it
        let sig = if let Some(signature) = &self.signature {
            signature
        } else {
            // The vote must be invalid, since it doesn't even have a signature
            return false;
        };

        // Ensure that the signature is valid, considering the vote's hash
        sig.verify(&*self.hash())
    }

    /// Hashes the contents of the vote, excluding any signature.
    pub fn hash(&self) -> HashPrim {
        // Copy the vote since we need to remove the signature from it to ensure validity
        let mut to_be_hashed = self.clone();
        to_be_hashed.signature = None;

        // Hash the vote's contents
        self::blake3::hash_slice(&bincode::serialize(&to_be_hashed).unwrap_or_default())
    }

    /// Derives an address from the signature associated with the vote.
    pub fn voter_address(&self) -> Option<Address> {
        // The vote must have a signature in order to derive a voter address from it
        if let Some(sig) = &self.signature {
            // Convert the signature's public key to an address
            sig.address().ok()
        } else {
            None
        }
    }
}

impl fmt::Display for Vote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // If the vote is in favor, express such an agreement as a string by saying "in favor of
        // prop"
        write!(
            f,
            "{}",
            if self.in_favor {
                format!("in favor of proposal {}", self.target_proposal)
            } else {
                format!("in opposition to proposal {}", self.target_proposal)
            }
        )
    }
}
