use super::{client::ClientBehavior, state::RuntimeEvent};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{
    gossipsub::{GossipsubEvent, Topic},
    swarm::NetworkBehaviourEventProcess,
};

/// A topic for all transactions in a network.
pub const TRANSACTIONS_TOPIC: &str = "transactions";

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<GossipsubEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming gossipsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, message: GossipsubEvent) {
        info!("esjoawejfoawejf");
        match message {
            GossipsubEvent::Message(peer_id, message_id, message) => {
                info!(
                    "received message from peer {} with id {}: {:?}",
                    peer_id, message_id, message
                );
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
                            .publish(&Topic::new(TRANSACTIONS_TOPIC.to_owned()), ser);

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
