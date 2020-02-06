use super::client::ClientBehavior;
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
