use super::{
    super::{
        core::sys::{
            proposal::{Operation, Proposal},
            system::System,
            vote::Vote,
        },
        crypto::hash::Hash,
        validator::{GraphBoundValidator, Validator},
    },
    client::ClientBehavior,
};
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    swarm::NetworkBehaviourEventProcess,
};
use num::{bigint::BigUint, CheckedDiv, Zero};
use std::sync::RwLockWriteGuard;

/// A topic for all proposals in a network.
pub const PROPOSALS_TOPIC: &str = "proposals";

/// A topic for all votes in a network.
pub const VOTES_TOPIC: &str = "votes";

impl NetworkBehaviourEventProcess<FloodsubEvent> for ClientBehavior {
    /// Wait for an incoming gossipsub message from a known peer. Handle it somehow.
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            // Print out the message's details
            debug!("Received message: {:?}", message);

            // if the message can't be attributed to a single topic, return
            if message.topics.is_empty() {
                return;
            }

            // If the message is a proposal message, handle it as such
            if message.topics[0].id() == PROPOSALS_TOPIC {
                debug!("Message is a proposal message; handling it as such");

                // Try to deserialize a proposal from the provided message data. If this fails, we'll want to print the error to stderr.
                let proposal: Proposal = match bincode::deserialize(&message.data) {
                    Ok(deserialized) => deserialized,
                    Err(e) => {
                        warn!("Failed to deserialize proposal received via pubsub: {}", e);

                        return;
                    }
                };

                // Get a writing lock on the client's runtime so that we can add the proposal
                let mut rt = match self.runtime.write() {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        warn!(
                            "Failed to obtain a writing lock on the client's runtime: {}",
                            e
                        );

                        return;
                    }
                };
                // Print out the proposal's details
                info!(
                    "Received new proposal to '{}' '{}': '{}' ({})",
                    proposal.proposal_data.operation,
                    proposal.proposal_data.param_name,
                    proposal.proposal_name,
                    proposal.proposal_id
                );

                // Copy the name of the parameter that the proposal will be changing so that we can vote on it.
                let param_name = proposal.proposal_data.param_name.clone();
                let id = proposal.proposal_id;
                let data = proposal.proposal_data.clone();

                // Add the proposal to the runtime
                rt.push_proposal(proposal);

                // If this is a proposal that we can automatically vote on, do it.
                if param_name == "ledger::transactions" {
                    // Get the data stored in the proposal
                    let tx_bytes = if let Operation::Append { value_to_append } = data.operation {
                        value_to_append
                    } else {
                        return;
                    };

                    // Derive a transaction from the data
                    let tx = if let Ok(deserialized) = bincode::deserialize(&tx_bytes) {
                        deserialized
                    } else {
                        return;
                    };

                    // The votes that we've generated for the proposal from each votinig account
                    let mut resultant_votes: Vec<Vote> = Vec::new();

                    // Print out the beginning voting process
                    info!("Automatically verifying, and voting in accordance to the result of the output of the chosen validator with {} accounts", self.voting_accounts.len());

                    // Vote for the proposal with each voting account
                    for i in 0..self.voting_accounts.len() {
                        // Try to get a keypair for the account that we can use to vote with
                        if let Ok(keypair) = self.voting_accounts[i].keypair() {
                            // Make a validator for the transaction
                            let validator = GraphBoundValidator::new(&rt.ledger);

                            // See if the transaction is valid or not
                            let reason = validator.transaction_is_valid(&tx);

                            // Make the vote
                            let vote = Vote::new(id, reason.is_ok(), keypair);

                            // Save the vote for later so we can publish it
                            resultant_votes.push(vote.clone());

                            // Register the vote
                            match rt.register_vote_for_proposal(id, &vote) {
                                Ok(_) => {
                                    info!(
                                        "Successfully submitted vote for proposal {}: {} because {}",
                                        id, vote.in_favor, if let Some(e) = reason.err() {format!("{}", e)} else {"transaction is valid".to_owned()});
                                }
                                Err(e) => warn!("Failed to vote for proposal {}: {}", id, e),
                            }
                        }
                    }

                    // Publish each of the votes that we collected from the unlocked
                    // accounts
                    if let Err(e) = publish_votes(resultant_votes, &mut self.gossipsub) {
                        warn!("Failed to publish votes: {}", e);
                    }
                }
            } else if message.topics[0].id() == VOTES_TOPIC {
                debug!("Message is a vote message; handling it as such");

                // Deserialize the vote that was sent to us via pubsub, encoded with bincode
                let vote: Vote = match bincode::deserialize(&message.data) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to deserialize vote received via pubsub: {}", e);

                        return;
                    }
                };

                // If the vote isn't valid, we must not continue
                if !vote.valid() {
                    warn!("Received invalid vote: {}", vote.hash());

                    return;
                }

                // Eventually, we'll want to register this vote in the runtime
                let mut rt = if let Ok(rt) = self.runtime.write() {
                    rt
                } else {
                    warn!("Failed to obtain a lock on the client's runtime. Aborting incoming vote registration process.");

                    return;
                };

                // The proposal that the vote is in favor or against should exist in the runtime.
                // Otherwise, it is invalid.
                if !rt.pending_proposals.contains_key(&vote.target_proposal) {
                    warn!(
                        "Received invalid vote: {} (targets an unknown proposal)",
                        vote.hash()
                    );

                    return;
                }

                // Collect metadata regarding the vote so that we can alert the user of the vote in
                // the future
                let vote_value: String = format!("{}", vote);
                let vote_hash = vote.hash();

                // Register the vote, and log any errors that come up along the way
                if let Err(e) = rt.register_vote_for_proposal(vote.target_proposal, &vote) {
                    warn!("Failed to register vote {}: {}", vote_hash, e);

                    return;
                }

                // Log the success!
                info!(
                    "Received a new vote: {}; registered it with the runtime successfully",
                    vote_value
                );

                // Try to clear the proposal
                if potentially_clear_proposal(rt, &vote.target_proposal) {
                    info!("Successfully cleared proposal {}!", vote.target_proposal);
                } else {
                    debug!("Proposal {} is not mature enough...", vote.target_proposal);
                }
            }
        }
    }
}

/// Publishes each of the provided votes via pubsub, using the provided floodsub adapter.
/// This helper method terminates execution on the first error.
///
/// # Arguments
///
/// * `votes` - The votes that should be published
/// * `adapter` - The floodsub instance that the votes will be published with
pub(crate) fn publish_votes(votes: Vec<Vote>, adapter: &mut Floodsub) -> bincode::Result<()> {
    for vote in votes.iter() {
        match bincode::serialize(vote) {
            Ok(serialized) => adapter.publish(Topic::new(VOTES_TOPIC), serialized),
            Err(e) => {
                warn!(
                    "Failed to serialize and publish vote {}: {}",
                    vote.hash(),
                    e
                );

                return Err(e);
            }
        }
    }

    Ok(())
}

/// Attempts to execute the given proposal, on the condition that it has enough vote weight to
/// pass.
///
/// # Arguments
///
/// * `runtime` - The runtime context that should be used to determine whether or not the proposal
/// may be executed
/// * `proposal` - The proposal that should be executed
pub(crate) fn potentially_clear_proposal(
    mut runtime: RwLockWriteGuard<System>,
    proposal: &Hash,
) -> bool {
    // Get the # of coins that the proposer must have, at least, in order to execute it
    let acceptable_majority = if let Some(maj) = runtime
        .ledger
        .overall_issuance()
        .checked_div(&BigUint::from(2 as u8))
    {
        maj
    } else {
        return false;
    };

    // In order to execute the proposal, we must have a 1/2 majority
    if runtime
        .get_coins_in_support_of(proposal)
        .to_biguint()
        .unwrap_or_else(BigUint::zero)
        >= acceptable_majority
    {
        // If the proposal is invalid, don't execute it, but clear it, nonetheless
        if !runtime.validate_proposal(proposal) {
            runtime.pending_proposals.remove(proposal);

            return false;
        }

        // Execute the proposal
        return match runtime.execute_proposal(*proposal) {
            Ok(()) => true,
            Err(e) => {
                warn!("Failed to execute proposal {}: {}", proposal, e);

                false
            }
        };
    } else if (-runtime.get_coins_in_support_of(proposal))
        .to_biguint()
        .unwrap_or_else(BigUint::zero)
        >= acceptable_majority
    {
        return runtime.pending_proposals.remove(proposal).is_some();
    }

    false
}
