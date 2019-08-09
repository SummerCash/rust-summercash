use super::super::types::signature; // Import the signature primitive

/// A binary, signed vote regarding a particular proposal.
pub struct Vote {
    /// The hash of the target proposal
    pub target_proposal: hash::Hash,
    /// The signature of the voter
    pub signature: signature::Signature,
}