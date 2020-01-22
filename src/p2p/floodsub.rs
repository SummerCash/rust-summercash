use super::client::ClientBehavior;
use futures::{AsyncRead, AsyncWrite};
use libp2p::{floodsub::FloodsubEvent, swarm::NetworkBehaviourEventProcess};

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<FloodsubEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming floodsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, _message: FloodsubEvent) {
        // TODO: Unimplemented
    }
}
