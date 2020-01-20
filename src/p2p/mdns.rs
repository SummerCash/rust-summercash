use super::client::ClientBehavior;
use futures::{AsyncRead, AsyncWrite};
use libp2p::{mdns::MdnsEvent, swarm::NetworkBehaviourEventProcess};

/// Discovery via mDNS events.
impl<'a, TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<MdnsEvent> for ClientBehavior<'a, TSubstream>
{
    /// Wait for an incoming mDNS message from a potential peer. Add them to the local registry if the connection succeeds.
    fn inject_event(&mut self, event: MdnsEvent) {
        // Check what kind of packet the peer has sent us, and, from there, decide what we want to do with it.
        match event {
            MdnsEvent::Discovered(list) =>
            // Go through each of the peers we were able to connect to, and add them to the localized node registry
            {
                for (peer, addr) in list {
                    // Log the discovered peer to stdout
                    debug!("Received mDNS 'alive' confirmation from peer: {}", peer);

                    // Register the discovered peer in the localized KAD DHT service instance
                    self.kad_dht.add_address(&peer, addr);

                    // Register the discovered peer in the localized pubsub service instance
                    self.floodsub.add_node_to_partial_view(peer)
                }
            }
            MdnsEvent::Expired(list) =>
            // Go through each of the peers we were able to connect to, and remove them from the localized node registry
            {
                for (peer, _) in list {
                    if self.mdns.has_node(&peer) {
                        // Log the peer that will be removed
                        info!("Peer {} dead; removing", peer);

                        // Oops, rent is up, and the bourgeoisie haven't given up their power. I guess it's time to die, poor person. Sad proletariat.
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}
