use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library

use std::collections; // Import the collections library

use chrono; // Import time library

use num::{bigint::BigUint, Zero}; // Add support for large unsigned integers

use bincode;
use serde::{Deserialize, Serialize}; // Import serde serialization
use serde_json; // Import serde json // Import serde bincode

use super::receipt; // Import receipt types
use super::signature; // Import signature type
use super::state; // Import the state entry types

use super::super::super::{common::address, crypto::blake3, crypto::hash}; // Import the hash & address modules

/// An error encountered while signing a tx.
#[derive(Debug, Fail)]
pub enum SignatureError {
    #[fail(
        display = "transaction sender address does not match public key hash: {}",
        address_hex
    )]
    InvalidAddressPublicKeyCombination {
        address_hex: String, // The hex-encoded sender address
    },
}

/// A transaction between two different addresses on the SummerCash network.
#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    /// The contents of the transaction
    pub transaction_data: TransactionData,
    /// The hash of the transaction
    pub hash: hash::Hash,
    /// The transaction's signature
    pub signature: Option<signature::Signature>,
    /// The address of the deployed contract (if applicable)
    pub deployed_contract_address: Option<address::Address>,
    /// Whether or not this transaction creates a contract
    pub contract_creation: bool,
    /// Whether or not this transaction is the network genesis
    pub genesis: bool,
}

/// A container representing the contents of a transaction.
#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionData {
    /// The index of the transaction in the sender's set of txs
    pub nonce: u64,
    /// The sender of the transaction
    pub sender: address::Address,
    /// The recipient of the transaction
    pub recipient: address::Address,
    /// The amount of finks sent along with the Transaction
    pub value: BigUint,
    /// The data sent to the transaction recipient (i.e. contract call bytecode)
    pub payload: Vec<u8>,
    /// The hashes of the transaction's parents
    pub parents: Vec<hash::Hash>,
    /// The list of resolved parent receipts
    pub parent_receipts: Option<receipt::ReceiptMap>,
    /// The hash of the combined parent state
    pub parent_state_hash: Option<hash::Hash>,
    /// The transaction's timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/* BEGIN EXPORTED METHODS */

impl TransactionData {
    /// Serialize a given TransactionData instance into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap() // Serialize
    }
}

/// Implement a set of transaction helper methods.
impl Transaction {
    /// Initialize a new transaction instance from a given set of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::transaction; // Import the transaction library
    /// use summercash::{common::address, crypto::hash}; // Import the address library
    ///
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = &mut transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    /// ```
    pub fn new(
        nonce: u64,
        sender: address::Address,
        recipient: address::Address,
        value_finks: BigUint,
        payload: &[u8],
        parents: Vec<hash::Hash>,
    ) -> Transaction {
        let transaction_data: TransactionData = TransactionData {
            nonce,                         // Set nonce
            sender,                        // Set sender
            recipient,                     // Set recipient
            value: value_finks,            // Set value (in finks)
            payload: payload.to_vec(),     // Set payload
            parents,                       // Set parents
            parent_receipts: None,         // Set parent receipts
            parent_state_hash: None,       // Set parent state hash
            timestamp: chrono::Utc::now(), // Set timestamp
        }; // Initialize transaction data

        let mut transaction_data_bytes = vec![0; transaction_data.to_bytes().len()]; // Initialize transaction data buffer

        transaction_data_bytes.clone_from_slice(transaction_data.to_bytes().as_slice()); // Copy into buffer

        Transaction {
            transaction_data, // Set transaction data
            hash: blake3::hash_slice(transaction_data_bytes.as_slice()),
            signature: None, // Set signature
            deployed_contract_address: None,
            contract_creation: false, // Set does create contract
            genesis: false,           // Set is genesis
        }
    }

    /// Verify the signature attached to a transaction.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::transaction; // Import the transaction library
    /// use summercash::{common::address, crypto::hash}; // Import the address library
    ///
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = &mut transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    /// transaction::sign_transaction(sender_keypair, tx).unwrap(); // Sign tx
    ///
    /// let sig_valid = tx.verify_signature(); // Verify signature
    /// ```
    pub fn verify_signature(&self) -> bool {
        match &self.signature {
            None => false,                                    // Nil signature can't be valid
            Some(signature) => signature.verify(&*self.hash), // Verify signature
        }
    }

    /// Execute creates a new state entry from the current transaction, regardless of network state. TODO: Support contracts
    ///
    /// # Example
    ///
    /// ```
    /// extern crate num; // Link num library
    /// extern crate rand; // Link rand library
    ///
    /// use num::traits::FromPrimitive; // Allow overloading of from_i64()
    /// use num::bigint::BigUint; // Add support for large unsigned integers
    ///
    /// use rand::rngs::OsRng; // Import the os's rng
    ///
    /// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
    ///
    /// use summercash::core::types::transaction; // Import the transaction library
    /// use summercash::{common::address, crypto::hash}; // Import the address library
    ///
    /// let mut csprng = OsRng{}; // Generate source of randomness
    ///
    /// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
    /// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
    ///
    /// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
    /// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
    ///
    /// let tx = &mut transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
    /// let resulting_state_entry = tx.execute(None); // Must specify a previous state entry if this is not the genesis transaction
    /// ```
    pub fn execute(&self, prev_entry: Option<state::Entry>) -> state::Entry {
        match prev_entry {
            Some(entry) => {
                // Execute the transaction, but with no entry data, since there isn't anything in the entry in the first place
                if entry.data.balances.is_empty() {
                    return self.execute(None);
                }

                let mut balances: collections::HashMap<String, BigUint> = entry.data.balances; // Initialize balances map

                balances.insert(
                    self.transaction_data.sender.to_str(),
                    balances
                        .get(&self.transaction_data.sender.to_str())
                        .unwrap_or(&BigUint::zero())
                        - self.transaction_data.value.clone(),
                ); // Subtract transaction value from sender balance
                balances.insert(
                    self.transaction_data.recipient.to_str(),
                    balances
                        .get(&self.transaction_data.recipient.to_str())
                        .unwrap_or(&BigUint::zero())
                        + self.transaction_data.value.clone(),
                ); // Add transaction value to recipient balance

                state::Entry::new(balances) // Return state entry
            }
            None => {
                let mut balances: collections::HashMap<String, BigUint> =
                    collections::HashMap::new(); // Initialize balance map
                balances.insert(
                    self.transaction_data.recipient.to_str(),
                    self.transaction_data.value.clone(),
                ); // Set recipient balance to tx value

                state::Entry::new(balances) // Return state entry
            }
        }
    }

    /// Serialize a given transaction instance into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap() // Serialize
    }

    /// Deserialize a transaction instance from a given byte vector.
    pub fn from_bytes(b: &[u8]) -> Transaction {
        bincode::deserialize(b).unwrap() // Deserialize
    }
}

/// Sign a given transaction with the provided ed25519 keypair.
///
/// # Example
///
/// ```
/// extern crate num; // Link num library
/// extern crate rand; // Link rand library
///
/// use num::traits::FromPrimitive; // Allow overloading of from_i64()
/// use num::bigint::BigUint; // Add support for large unsigned integers
///
/// use rand::rngs::OsRng; // Import the os's rng
///
/// use ed25519_dalek::Keypair; // Import the edwards25519 digital signature library
///
/// use summercash::core::types::transaction; // Import the transaction library
/// use summercash::{common::address, crypto::hash}; // Import the address library
///
/// let mut csprng = OsRng{}; // Generate source of randomness
///
/// let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
/// let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair
///
/// let sender = address::Address::from_key_pair(&sender_keypair); // Derive sender from sender key pair
/// let recipient = address::Address::from_key_pair(&recipient_keypair); // Derive recipient from recipient key pair
///
/// let tx = &mut transaction::Transaction::new(0, sender, recipient, BigUint::from_i64(0).unwrap(), b"test transaction payload", vec![hash::Hash::new(vec![0; hash::HASH_SIZE])]); // Initialize transaction
/// transaction::sign_transaction(sender_keypair, tx).unwrap(); // Sign tx
/// ```
pub fn sign_transaction(
    keypair: Keypair,
    transaction: &mut Transaction,
) -> Result<(), SignatureError> {
    let derived_sender_address = address::Address::from_key_pair(&keypair); // Derive sender address from key pair

    if transaction.transaction_data.sender != derived_sender_address {
        // Check is not sender
        return Err(SignatureError::InvalidAddressPublicKeyCombination {
            address_hex: derived_sender_address.to_str(),
        }); // Return error in result
    }

    let signature = signature::Signature {
        public_key: keypair.public,
        signature: keypair.sign(&*transaction.hash),
    }; // Initialize signature

    transaction.signature = Some(signature); // Set signature

    Ok(()) // Everything's good, right? I mean, it's not like anyone ever asks or anything. But then, again, in the end, does it really matter? I suppose from the viewpoint that our idea of existence is based purely on perception, this notion would in fact be correct.
}

/* END EXPORTED METHODS */

#[cfg(test)]
mod tests {
    use super::*; // Import names from the parent module

    use rand::rngs::OsRng; // Import the os's rng

    use num::BigRational; // Import the big rational type

    use std::{str, str::FromStr}; // Let the bigint library implement from_str

    use super::super::super::super::common::fink; // Import the fink conversion utility

    #[test]
    fn test_new() {
        let mut csprng = OsRng {}; // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let transaction = Transaction::new(
            0,
            address::Address::from_key_pair(&sender_keypair),
            address::Address::from_key_pair(&recipient_keypair),
            fink::convert_smc_to_finks(BigRational::from_str("10/1").unwrap()),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize transaction

        assert_eq!(
            str::from_utf8(transaction.transaction_data.payload.as_slice()).unwrap(),
            "test transaction payload"
        ); // Ensure payload intact
    }

    #[test]
    fn test_sign_transaction() {
        let mut csprng = OsRng {}; // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let transaction = &mut Transaction::new(
            0,
            address::Address::from_key_pair(&sender_keypair),
            address::Address::from_key_pair(&recipient_keypair),
            fink::convert_smc_to_finks(BigRational::from_str("10/1").unwrap()),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize transaction

        sign_transaction(sender_keypair, transaction).unwrap(); // Sign transaction
    }

    #[test]
    fn test_verify_transaction_signature() {
        let mut csprng = OsRng {}; // Generate source of randomness

        let sender_keypair: Keypair = Keypair::generate(&mut csprng); // Generate sender key pair
        let recipient_keypair: Keypair = Keypair::generate(&mut csprng); // Generate recipient key pair

        let transaction = &mut Transaction::new(
            0,
            address::Address::from_key_pair(&sender_keypair),
            address::Address::from_key_pair(&recipient_keypair),
            fink::convert_smc_to_finks(BigRational::from_str("10/1").unwrap()),
            b"test transaction payload",
            vec![hash::Hash::new(vec![0; hash::HASH_SIZE])],
        ); // Initialize transaction

        sign_transaction(sender_keypair, transaction).unwrap(); // Sign transaction

        assert!(transaction.verify_signature()); // Ensure signature valid
    }
}
