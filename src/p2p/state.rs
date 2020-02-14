/// This module implements a state-ful NetworkBehaviour that essentially acts as a shell for the `Runtime` type inside
/// a struct that is using a derived NetworkBehaviour.
use super::{
    super::core::sys::{proposal::Proposal, system::System},
    client::CommunicationError,
};
use core::task::{Context, Poll};
use futures::{AsyncRead, AsyncWrite};

use std::{
    io,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

use libp2p::{
    core::ConnectedPoint,
    ping::protocol::Ping,
    swarm::NetworkBehaviour,
    swarm::{
        protocols_handler::{
            IntoProtocolsHandler, KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent,
            ProtocolsHandlerUpgrErr, SubstreamProtocol,
        },
        NetworkBehaviourAction, PollParameters,
    },
    InboundUpgrade, Multiaddr, OutboundUpgrade, PeerId,
}; // Import the libp2p library

/// A behavior for the Runtime network primitive.
pub struct RuntimeBehavior<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> {
    pub runtime: Arc<RwLock<System>>,

    stream: PhantomData<TSubstream>,

    ledger_ctx: Arc<AtomicBool>,
}

/// Represents a generic behavioral event emitted by the state contained inside a client.
pub enum RuntimeEvent {
    /// An event representing a new set of proposals that has been added to the publishing queue
    QueuedProposals(Vec<Proposal>),
}

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> RuntimeBehavior<TSubstream> {
    /// Gets a writing lock to the state's system.
    pub fn write(&mut self) -> Result<RwLockWriteGuard<System>, failure::Error> {
        // Try to get a writing lock on the runtime
        if let Ok(rt) = self.runtime.write() {
            // Return the runtime
            Ok(rt)
        } else {
            // Return a mutex failure error
            Err(CommunicationError::MutexFailure.into())
        }
    }

    /// Gets a reading lock on the state's system.
    pub fn read(&mut self) -> Result<RwLockReadGuard<System>, failure::Error> {
        // Try to get a reading lock on the runtime
        if let Ok(rt) = self.runtime.read() {
            // Return the runtime
            Ok(rt)
        } else {
            // Return a mutex failure error
            Err(CommunicationError::MutexFailure.into())
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> NetworkBehaviour
    for RuntimeBehavior<TSubstream>
{
    // This behaviour isn't really doing anything, so we don't need to spec out any types
    type ProtocolsHandler = Handler<TSubstream>;

    type OutEvent = RuntimeEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        Handler::<TSubstream>(PhantomData)
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connected(&mut self, _peer_id: PeerId, _endpoint: ConnectedPoint) {}

    fn inject_disconnected(&mut self, _peer_id: &PeerId, _endpoint: ConnectedPoint) {}

    fn inject_node_event(
        &mut self,
        _peer_id: PeerId,
        _event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent,
    ) {
    }

fn poll(&mut self, _cx: &mut Context, _params: &mut impl PollParameters) -> Poll<NetworkBehaviourAction<<<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent, Self::OutEvent>>{
        match self.ledger_ctx.load(Ordering::SeqCst) {
            true => {
                // Get a reading reference to the runtime instance so that we can load a list of
                // new proposals
                if let Ok(rt) = self.runtime.read() {
                    info!(
                        "Discovered {} new proposals to publish; alerting the swarm...",
                        rt.localized_proposals.values().len()
                    );

                    Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                        RuntimeEvent::QueuedProposals(
                            rt.localized_proposals.values().cloned().collect(),
                        ),
                    ))
                } else {
                    Poll::Pending
                }
            }
            false => Poll::Pending,
        }
    }
}

impl<'a, TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> RuntimeBehavior<TSubstream> {
    /// Initializes a new RuntimeBehavior with the given runtime reference.
    pub fn new(runtime: std::sync::Arc<std::sync::RwLock<System>>) -> Self {
        // Get reference to the provided runtime's current new_tx context variable. This is used to
        // periodically publish transactions.
        let ledger_ctx = if let Ok(rt) = runtime.read() {
            rt.get_state_ref()
        } else {
            // We'll just use an empty atomic bool for now
            Arc::new(AtomicBool::new(false))
        };

        // Initialize a new runtime behavior with the given runtime reference
        Self {
            runtime,
            stream: PhantomData,
            ledger_ctx,
        }
    }
}

/// A generic, non-functional handler for this "protocol".
pub struct Handler<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>(
    PhantomData<TSubstream>,
);

impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static> ProtocolsHandler
    for Handler<TSubstream>
{
    type InEvent = ();
    type OutEvent = RuntimeEvent;
    type Error = io::Error;
    type Substream = TSubstream;
    type InboundProtocol = Ping;
    type OutboundProtocol = Ping;
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        SubstreamProtocol::new(Ping)
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        _protocol: <Self::InboundProtocol as InboundUpgrade<Self::Substream>>::Output,
    ) {
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        _protocol: <Self::OutboundProtocol as OutboundUpgrade<Self::Substream>>::Output,
        _info: Self::OutboundOpenInfo,
    ) {
    }

    fn inject_event(&mut self, _event: Self::InEvent) {}

    fn inject_dial_upgrade_error(
        &mut self,
        _info: Self::OutboundOpenInfo,
        _error: ProtocolsHandlerUpgrErr<
            <Self::OutboundProtocol as OutboundUpgrade<Self::Substream>>::Error,
        >,
    ) {
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    fn poll(
        &mut self,
        _cx: &mut Context,
    ) -> Poll<
        ProtocolsHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::OutEvent,
            Self::Error,
        >,
    > {
        Poll::Pending
    }
}
