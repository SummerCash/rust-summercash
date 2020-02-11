use ed25519_dalek; // Import the edwards25519 digital signature library

use bincode; // Import serde bincode
use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::crypto::hash; // Import the hash primitive
use super::super::types::signature; // Import the signature primitive

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

        if let Ok(serialized_vote) = bincode::serialize(&vote) {
            // Serialize vote
            vote.signature = Some(signature::Signature {
                public_key_bytes: bincode::serialize(&signature_keypair.public).unwrap_or_default(),
                signature_bytes: bincode::serialize(
                    &signature_keypair.sign(serialized_vote.as_slice()),
                )
                .unwrap_or_default(),
            }); // Set signature
        }

        vote // Return initialized vote
    }
}
