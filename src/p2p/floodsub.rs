use super::{
    super::{
        core::sys::{
            proposal::{Operation, Proposal},
            vote::Vote,
        },
        validator::{GraphBoundValidator, Validator},
    },
    client::ClientBehavior,
    state::RuntimeEvent,
};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{
    floodsub::{FloodsubEvent, TopicBuilder},
    swarm::NetworkBehaviourEventProcess,
};
use std::error::Error;

/// A topic for all proposals in a network.
pub const PROPOSALS_TOPIC: &str = "proposals";

/// A topic for all votes in a network.
pub const VOTES_TOPIC: &str = "votes";

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<FloodsubEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming gossipsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, message: FloodsubEvent) {
        match message {
            FloodsubEvent::Message(message) => {
                // Print out the message's details
                debug!("Received message: {:?}", message);

                // if the message can't be attributed to a single topic, return
                if message.topics.len() == 0 {
                    return;
                }

                // If the message is a proposal message, handle it as such
                if message.topics[0] == *TopicBuilder::new(PROPOSALS_TOPIC).build().hash() {
                    // Try to deserialize a proposal from the provided message data. If this fails, we'll want to print the error to stderr.
                    let proposal: Proposal = match bincode::deserialize(&message.data) {
                        Ok(deserialized) => deserialized,
                        Err(e) => {
                            warn!("Failed to deserialize proposal received via pubsub: {}", e);

                            return;
                        }
                    };

                    // Get a writing lock on the client's runtime so that we can add the proposal
                    let mut rt = match self.runtime.write() {
                        Ok(runtime) => runtime,
                        Err(e) => {
                            warn!(
                                "Failed to obtain a writing lock on the client's runtime: {}",
                                e
                            );

                            return;
                        }
                    };
                    // Print out the proposal's details
                    info!(
                        "Received new proposal to '{}' '{}': '{}' ({})",
                        proposal.proposal_data.operation,
                        proposal.proposal_data.param_name,
                        proposal.proposal_name,
                        proposal.proposal_id
                    );

                    // Copy the name of the parameter that the proposal will be changing so that we can vote on it.
                    let param_name = proposal.proposal_data.param_name.clone();
                    let id = proposal.proposal_id.clone();
                    let data = proposal.proposal_data.clone();

                    // Add the proposal to the runtime
                    rt.push_proposal(proposal);

                    // If this is a proposal that we can automatically vote on, do it.
                    if param_name == "ledger::transactions" {
                        match data.operation {
                            Operation::Append {
                                value_to_append: tx_bytes,
                            } => {
                                // Derive a transaction from the data
                                let tx = if let Ok(deserialized) = bincode::deserialize(&tx_bytes) {
                                    deserialized
                                } else {
                                    return;
                                };

                                // The votes that we've generated for the proposal from each votinig account
                                let mut resultant_votes: Vec<Vote> = Vec::new();

                                // Print out the beginning voting process
                                info!("Automatically verifying, and voting in accordance to the result of the output of the chosen validator with {} accounts", self.voting_accounts.len());

                                // Vote for the proposal with each voting account
                                for i in 0..self.voting_accounts.len() {
                                    // Try to get a keypair for the account that we can use to vote with
                                    if let Ok(keypair) = self.voting_accounts[i].keypair() {
                                        // Make a validator for the transaction
                                        let validator = GraphBoundValidator::new(&rt.ledger);

                                        // See if the transaction is valid or not
                                        let reason = validator.transaction_is_valid(&tx);

                                        let vote = Vote::new(id, reason.is_ok(), keypair); // Make the vote

                                        // Save the vote for later so we can publish it
                                        resultant_votes.push(vote.clone());

                                        // Register the vote
                                        match rt.register_vote_for_proposal(id, vote.clone()) {
                                            Ok(_) => {
                                                info!(
                                                "Successfully submitted vote for proposal {}: {} because {}",
                                                id, vote.in_favor, if let Some(e) = reason.err() {let e_err = e.compat(); format!("{}", e_err.description())} else {"transaction is valid".to_owned()});
                                            }
                                            Err(e) => {
                                                warn!("Failed to vote for proposal {}: {}", id, e)
                                            }
                                        }
                                    }
                                }

                                // Publish each of the votes. This can't be done in a single loop, as rt is borrowed from self.
                                for vote in resultant_votes {
                                    // Serialize the vote and publish it
                                    if let Ok(serialized) = bincode::serialize(&vote) {
                                        self.gossipsub.publish(
                                            TopicBuilder::new(VOTES_TOPIC).build(),
                                            serialized,
                                        );
                                    }
                                }
                            }
                            _ => {
                                return;
                            }
                        };
                    }
                }
            }
            _ => (),
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<RuntimeEvent> for ClientBehavior<TSubstream>
{
    /// Handle a pseudo-event from an executor. While this event might look like it's coming from a network peer, it
    /// is really a command to publish a transaction from an external source (i.e. RPC).
    fn inject_event(&mut self, message: RuntimeEvent) {
        match message {
            // A new transaction has been found that a user would like to publish. Publish it.
            RuntimeEvent::QueuedProposals(props) => {
                // Alert the user of the new proposals
                info!("Publishing {} new proposals...", props.len());

                // Get a mutable reference to the client's runtime so that we can update the list
                // of pending proposals once we publish a prop.
                let mut rt = if let Ok(runtime) = self.runtime.write() {
                    runtime
                } else {
                    return;
                };

                // Publish each proposal
                for (i, prop) in props.iter().enumerate() {
                    // Try to serialize the proposal. If this succeeds, we can try to publish the
                    // proposal.
                    if let Ok(ser) = bincode::serialize(&prop) {
                        // We've got a serialized proposal; publish it
                        self.gossipsub
                            .publish(&TopicBuilder::new(PROPOSALS_TOPIC.to_owned()).build(), ser);

                        // Propose the proposal
                        match rt.propose_proposal(prop.proposal_id) {
                            Ok(_) => {
                                info!("Successfully proposed proposal {}: {}", i, prop.proposal_id)
                            }
                            Err(e) => {
                                warn!("Failed to propose proposal {}: {}", prop.proposal_id, e)
                            }
                        }
                    } else {
                        warn!(
                            "Failed to serialize proposal with hash: {}",
                            prop.proposal_id
                        );
                    }
                }

                // Clear the runtime of all pending local proposals
                rt.clear_localized_proposals();
            }
        }
    }
}
