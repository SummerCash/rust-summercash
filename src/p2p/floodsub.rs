use super::{super::core::sys::{proposal::Proposal, vote::Vote}, client::ClientBehavior, state::RuntimeEvent};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{
    floodsub::{FloodsubEvent, TopicBuilder},
    swarm::NetworkBehaviourEventProcess,
};

/// A topic for all proposals in a network.
pub const PROPOSALS_TOPIC: &str = "proposals";

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

                    // Add the proposal to the runtime
                    rt.push_proposal(proposal);

                    // If this is a proposal that we can automatically vote on, do it.
                    if param_name == "ledger::transactions" {
                        let vote = Vote::new(id, )
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
                // Get a mutable reference to the client's runtime so that we can update the list
                // of pending proposals once we publish a prop.
                let mut rt = if let Ok(runtime) = self.runtime.write() {
                    runtime
                } else {
                    return;
                };

                // Publish each proposal
                for prop in props {
                    // Try to serialize the proposal. If this succeeds, we can try to publish the
                    // proposal.
                    if let Ok(ser) = bincode::serialize(&prop) {
                        // We've got a serialized proposal; publish it
                        self.gossipsub
                            .publish(&TopicBuilder::new(PROPOSALS_TOPIC.to_owned()).build(), ser);

                        // Propose the proposal
                        match rt.propose_proposal(prop.proposal_id) {
                            Ok(_) => (),
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
