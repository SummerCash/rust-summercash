use super::client::ClientBehavior;
use futures::{AsyncRead, AsyncWrite};
use libp2p::{mdns::MdnsEvent, swarm::NetworkBehaviourEventProcess};

/// Discovery via mDNS events.
impl
    NetworkBehaviourEventProcess<MdnsEvent> for ClientBehavior
{
    /// Wait for an incoming mDNS message from a potential peer. Add them to the local registry if the connection succeeds.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, addr) in list {
                    // Log the discovered peer to stdout
                    debug!("Received mDNS 'alive' confirmation from peer: {}", peer);

                    // Register the discovered peer in the localized KAD DHT service instance
                    self.kad_dht.add_address(&peer, addr);

                    // Register the node for floodsub
                    self.gossipsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _addr) in list {
                    // Log the discovered peer to stdout
                    debug!("mDNS peer dead: {}", peer);

                    // Register the node for floodsub
                    self.gossipsub.remove_node_from_partial_view(&peer);
                }
            }
        }
    }
}
