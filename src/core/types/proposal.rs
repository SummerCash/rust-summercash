/// A proposal regarding a network-wide action.
pub struct Proposal {
    /// The type of the proposal
    pub proposal_type: ProposalType,
}

/// The manner in which a particular atomic event should be treated.
pub enum Operation {
    /// Make a minor change, or revision to a particular attribute or event
    Amend { amended_value: Vec<u8> },
    /// Remove a particular attribute or event from the network's shared memory
    Remove,
    /// Add a value to a particular attribute or set of events
    Append { value_to_append: Vec<u8> },
}

/// The type of a proposal (e.g. ProtocolParamChange)
pub enum ProposalType {
    ProtocolParam {
        /// The name of the parameter to modify
        param_name: String,
        /// The manner in which to modify the operation (i.e. "")
        operation: Operation,
    },
}
