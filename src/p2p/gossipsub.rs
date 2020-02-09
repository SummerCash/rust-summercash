use super::{client::ClientBehavior, state::RuntimeEvent};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{gossipsub::GossipsubEvent, swarm::NetworkBehaviourEventProcess};

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<GossipsubEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming gossipsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, _message: GossipsubEvent) {
        // TODO: Unimplemented
    }
}

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<RuntimeEvent> for ClientBehavior<TSubstream>
{
    /// Handle a pseudo-event from an executor. While this event might look like it's coming from a network peer, it
    /// is really a command to publish a transaction from an external source (i.e. RPC).
    fn inject_event(&mut self, message: RuntimeEvent) {
        match message {
            // A new transaction has been found that a user would like to publish
            RuntimeEvent::QueuedProposal(prop) => {}
        }
    }
}
