use super::client::ClientBehavior;
use libp2p::{identify::IdentifyEvent, ping::PingEvent, swarm::NetworkBehaviourEventProcess};

/// Network liveliness enforcement via ping and identification swarm services.
impl NewtorkBehaviourEventProcess<IdentifyEvent> for ClientBehavior {}
