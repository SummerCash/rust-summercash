use super::client::ClientBehavior;
use futures::{AsyncRead, AsyncWrite};
use libp2p::{mdns::MdnsEvent, swarm::NetworkBehaviourEventProcess};

/// Discovery via mDNS events.
impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<MdnsEvent> for ClientBehavior<TSubstream>
{
    /// Wait for an incoming mDNS message from a potential peer. Add them to the local registry if the connection succeeds.
    fn inject_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(list) = event {
            for (peer, addr) in list {
                // Log the discovered peer to stdout
                debug!("Received mDNS 'alive' confirmation from peer: {}", peer);

                // Register the discovered peer in the localized KAD DHT service instance
                self.kad_dht.add_address(&peer, addr);
            }
        }
    }
}
