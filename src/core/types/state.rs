use std::collections; // Import the stdlib collections library

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::{crypto::blake3, crypto::hash}; // Import the hash modules

use num::bigint::BigUint; // Add support for large unsigned integers

/// The state at a particular point in time.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Entry {
    /// Body of the state entry
    pub data: EntryData,

    /// Hash of the state entry
    pub hash: hash::Hash,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EntryData {
    /// Balances of every account at a certain point in time
    pub balances: collections::HashMap<String, BigUint>,

    /// The last recorded index of each account
    pub nonces: collections::HashMap<String, u64>,
}

/// Implement a set of state entry serialization helper methods.
impl EntryData {
    /// Serialize a given EntryData instance into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap() // Serialize
    }
}

/// Implement a set of state helper methods.
impl Entry {
    /// Initialize a new Entry instance.
    pub fn new(
        nonces: collections::HashMap<String, u64>,
        balances: collections::HashMap<String, BigUint>,
    ) -> Entry {
        // Produce a reproducible hash of the state
        let state_hash = blake3::hash_slice(
            &bincode::serialize(&vec![
                bincode::serialize(&nonces.keys().collect::<Vec<&String>>().sort())
                    .unwrap_or_default(),
                bincode::serialize(&nonces.values().collect::<Vec<&u64>>().sort())
                    .unwrap_or_default(),
                bincode::serialize(&balances.keys().collect::<Vec<&String>>().sort())
                    .unwrap_or_default(),
                bincode::serialize(&balances.values().collect::<Vec<&BigUint>>().sort())
                    .unwrap_or_default(),
            ])
            .unwrap_or_default(),
        );

        let entry_data: EntryData = EntryData {
            balances, // Set balances
            nonces,   // Set nonces
        }; // Initialize entry data

        Entry {
            data: entry_data, // Set data
            hash: state_hash, // Set hash
        }
    }
}

/// Merge multiple state entires into one batch state entry.
pub fn merge_entries(entries: Vec<Entry>) -> Entry {
    let mut balances: collections::HashMap<String, BigUint> = collections::HashMap::new(); // Initialize balances map
    let mut nonces: collections::HashMap<String, u64> = collections::HashMap::new(); // Initialize a collections map

    for entry in entries {
        // Iterate through entries
        for (balance_addr, balance) in entry.data.balances.iter() {
            // Iterate through balances
            if balances.contains_key(balance_addr) {
                // Check already exists in balances
                let balance_difference = balance - balances.get(balance_addr).unwrap(); // Calculate balance difference

                let mut_balance = balances.get_mut(balance_addr).unwrap(); // Get mutable balance

                *mut_balance += balance_difference; // Set balance to difference between balance at old state and balance at tx
            } else {
                balances.insert(balance_addr.to_string(), balance.clone()); // Set balance
            }
        }

        // Synchronize both of the nonce storage locations
        for (nonce_addr, nonce) in entry.data.nonces.iter() {
            // Use the highest nonce for the account
            if *nonces.entry(nonce_addr.clone()).or_insert(*nonce) > *nonce {
                nonces.insert(nonce_addr.clone(), *nonce);
            }
        }
    }

    Entry::new(nonces, balances) // Return initialized state entry
}

#[cfg(test)]
mod tests {
    use super::*; // Import names from parent module

    use super::super::super::super::common::address; // Import the hash & address modules

    use crate::num::FromPrimitive; // Let the bigint library implement from_i64

    #[test]
    pub fn test_new() {
        let mut balances: collections::HashMap<String, BigUint> = collections::HashMap::new(); // Initialize balances hash map
        let nonces: collections::HashMap<String, u64> = collections::HashMap::new();

        balances.insert(
            blake3::hash_slice(b"test").to_str(),
            BigUint::from_i64(1).unwrap(),
        ); // Balance of 1 fink

        let entry: Entry = Entry::new(nonces, balances); // Initialize state entry

        assert_eq!(
            *entry
                .data
                .balances
                .get(&address::Address::new(blake3::hash_slice(b"test").to_vec()).to_str())
                .unwrap(),
            BigUint::from_i64(1).unwrap()
        ); // Ensure balance entry correctly written to state entry
    }
}
