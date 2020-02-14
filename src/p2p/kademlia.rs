use super::{
    super::{
        core::{
            sys::proposal::{Operation, Proposal, ProposalData},
            types::transaction::Transaction,
        },
        crypto::hash::Hash,
    },
    client::ClientBehavior,
    sync,
};

use futures::{AsyncRead, AsyncWrite};
use libp2p::{
    kad::{
        record::{Key, Record},
        GetRecordError, KademliaEvent, Quorum,
    },
    swarm::NetworkBehaviourEventProcess,
};

/// Network synchronization via KAD DHT events.
/// Synchronization of network proposals, for example, is done in this manner.
impl<TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static>
    NetworkBehaviourEventProcess<KademliaEvent> for ClientBehavior<TSubstream>
{
    // Wait for a peer to send us a kademlia event message. Once this happens, we can try to use the message for something (e.g. synchronization).
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            // The record was found successfully; print it
            KademliaEvent::GetRecordResult(Ok(result)) => {
                for Record { key, value, .. } in result.records {
                    // Handle different key types
                    match key.as_ref() {
                        b"ledger::transactions::root" => {
                            // Deserialize the root transaction hash from the given value
                            let root_hash: Hash = if let Ok(val) = bincode::deserialize(&value) {
                                // Alert the user that we've determined what the hash of the root tx is
                                info!(
                                    "Received the root transaction hash for the network: {}",
                                    val
                                );

                                val
                            } else {
                                return;
                            };

                            // Get a quorum to poll at least 50% of the network
                            let q: Quorum = self.active_subset_quorum();

                            // Get the actual root transaction, not just the hash, from the network
                            self.kad_dht.get_record(
                                &Key::new(&sync::transaction_with_hash_key(root_hash)),
                                q,
                            );
                        }

                        _ => {
                            // If the response is a transaction response, try deserializing the transaction, and doing something with it
                            if String::from_utf8_lossy(key.as_ref())
                                .contains("ledger::transactions::tx")
                            {
                                // Deserialize the transaction that the peer responded with
                                let tx: Transaction =
                                    if let Ok(val) = bincode::deserialize::<Transaction>(&value) {
                                        // Alert the user that we've obtained a copy of the tx
                                        info!(
                                            "Obtained a copy of a transaction with the hash: {}",
                                            val.hash.clone()
                                        );

                                        val
                                    } else {
                                        return;
                                    };

                                // Try to get a lock on the runtime so we can put the tx in the database
                                if let Ok(mut rt) = self.runtime.write() {
                                    // Make a proposal for the transaction, so we can execute it more effectively
                                    let proposal = Proposal::new(
                                        "sync_child".to_owned(),
                                        ProposalData::new(
                                            "ledger::transactions".to_owned(),
                                            Operation::Append {
                                                value_to_append: value,
                                            },
                                        ),
                                    );

                                    // The ID of the proposal. We need to copy this, since we'll move it into the system through registration
                                    let id = proposal.proposal_id;

                                    // Put the proposal in the system, so we can execute it
                                    rt.push_proposal(proposal);

                                    // Execute the proposal so it gets added to the dag
                                    match rt.execute_proposal(id) {
                                        Ok(_) => info!("Successfully executed transaction {}", id),
                                        Err(e) => warn!("Transaction execution failed: {}", e),
                                    }
                                }

                                // Get a quorum to poll at least 50% of the network
                                let q: Quorum = self.active_subset_quorum();

                                // Get the next hash in the dag
                                self.kad_dht
                                    .get_record(&Key::new(&sync::next_transaction_key(tx.hash)), q);
                            } else if String::from_utf8_lossy(key.as_ref())
                                .contains("ledger::transactions::next")
                            {
                                // Try to convert the raw bytes into an actual hash
                                let hash: Hash =
                                    if let Ok(val) = bincode::deserialize::<Hash>(&value) {
                                        info!(
                                            "Determined the next hash in the remote DAG: {}",
                                            val.clone()
                                        );

                                        val
                                    } else {
                                        return;
                                    };

                                // Get a quorum to poll at least 50% of the network
                                let q: Quorum = self.active_subset_quorum();

                                // Get the actual transaction corresponding to what we now know is the hash of such a transaction
                                self.kad_dht.get_record(
                                    &Key::new(&sync::transaction_with_hash_key(hash)),
                                    q,
                                );
                            }
                        }
                    }
                }
            }

            // An error occurred while fetching the record; print it
            KademliaEvent::GetRecordResult(Err(e)) => {
                // We'll want to handle different kinds of results from reading the DHT differently
                match e {
                    GetRecordError::NotFound { .. } => self.has_synchronized_dag = true,
                    _ => debug!("Failed to load record: {:?}", e),
                }
            }

            // The record was successfully set; print out the record name
            KademliaEvent::PutRecordResult(Ok(result)) => {
                // Since we've already published part of the DAG information, we don't really need to continue broadcasting
                self.should_broadcast_dag = false;

                // Print out the successful set operation
                info!(
                    "Set key successfully: {}",
                    String::from_utf8_lossy(result.key.as_ref())
                );
            }

            // An error occurred while fetching the record; print it
            KademliaEvent::PutRecordResult(Err(e)) => debug!("Failed to set key: {:?}", e),

            _ => {}
        }
    }
}
