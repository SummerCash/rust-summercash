use libp2p::Multiaddr; // Import libp2p

use bincode; // Import bincode

use super::super::{
    super::{core::sys::proposal, crypto::hash},
    client, message, network,
}; // Import the network, hash, proposal modules

use std::collections::HashMap; // Import the hashmap type

/// Download a copy of the network's list of proposals.
pub fn synchronize_for_network(
    network: network::Network,
    peers: Vec<Multiaddr>,
) -> Result<HashMap<hash::Hash, proposal::Proposal>, client::CommunicationError> {
    // Check has peers to bootstrap from
    if peers.len() != 0 {
        let header = message::Header::new(
            "proposals",
            message::Method::Get { summarize: false },
            vec![network],
        ); // Initialize header
        let message = message::Message::new(header, vec![]); // Initialize message

        let resp_result = client::broadcast_message_raw_with_response(message, peers); // Request the list of network proposals

        // Check for errors
        if let Ok(proposal_bytes) = resp_result {
            let proposal_list_result: Result<proposal::ProposalList, Box<bincode::ErrorKind>> =
                bincode::deserialize(&proposal_bytes); // Deserialize proposal list

            // Deserialize proposal list
            if let Ok(proposal_list) = proposal_list_result {
                let mut proposals: HashMap<hash::Hash, proposal::Proposal> = HashMap::new(); // Initialize proposals hash map

                // Iterate through proposals
                for proposal in proposal_list.proposals {
                    proposals.insert(proposal.proposal_id, proposal); // Add proposal to proposals map
                }

                Ok(proposals) // Return proposals
            } else {
                Err(client::CommunicationError::MessageSerializationFailure) // Return a serialization error, since we couldn't deserialize the list of proposals
            }
        } else {
            Err(resp_result.unwrap_err()) // Return error
        }
    } else {
        Err(client::CommunicationError::NoAvailablePeers) // Return an error since there are no available peers
    }
}
