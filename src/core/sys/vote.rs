use ed25519_dalek; // Import the edwards25519 digital signature library

use super::super::super::crypto::hash; // Import the hash primitive
use super::super::types::signature; // Import the signature primitive

/// A binary, signed vote regarding a particular proposal.
pub struct Vote {
    /// The hash of the target proposal
    pub target_proposal: hash::Hash,
    /// Whether the voter is in favor of the particular proposal or not
    pub in_favor: bool,
    /// The signature of the voter
    pub signature: signature::Signature,
}

/// Implement a set of voting helper methods.
impl Vote {
    /// Initialize and sign a new vote instance.
    pub fn new(proposal_id: hash::Hash, in_favor: bool, signature_keypair: ed25519_dalek::Keypair) -> Vote {
        Vote {
            target_proposal: proposal_id, // Set proposal ID
            in_favor: in_favor, // Set in favor of proposal
            signature: signature::Signature{public_key: signature_keypair.public, signature: signature_keypair.sign(&*proposal_id)}, // Set signature
        } // Return initialized vote
    }
}