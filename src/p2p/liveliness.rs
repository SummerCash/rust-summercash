use super::{super::core::sys::config, client::ClientBehavior, network::Network};
use libp2p::{identify::IdentifyEvent, ping::PingEvent, swarm::NetworkBehaviourEventProcess};

/// Network liveliness enforcement via identification swarm services.
impl NetworkBehaviourEventProcess<IdentifyEvent> for ClientBehavior {
    fn inject_event(&mut self, event: IdentifyEvent) {
        // We basically only have to handle the identification event if we're the ones sending it
        match event {
            IdentifyEvent::Received {
                peer_id,
                info,
                observed_addr: addr,
            } => {
                // If this peer is from a different network, or they're too old, but they're still trying to connect to
                // us, ban them.
                if <Network as From<String>>::from(info.protocol_version) != self.network
                    || !config::is_compatible_with_client(&info.agent_version)
                {
                    // Remove the peer from the client's perspective
                    self.remove_address(&peer_id);

                    return;
                }

                // Since the node is compatible, we can register them in each of our local views of
                // the network
                self.add_address(peer_id, addr);
            }
            _ => debug!("Received identification event: {:?}", event),
        }
    }
}

impl NetworkBehaviourEventProcess<PingEvent> for ClientBehavior {
    fn inject_event(&mut self, event: PingEvent) {
        match event.result {
            // Since our connection to the peer has basically cut out, we can remove them from our
            // view of the network
            Err(e) => {
                debug!("Removing peer {}: {}", event.peer, e);
                self.remove_address(&event.peer);
            }
            _ => debug!("Received ping event: {:?}", event),
        }
    }
}
