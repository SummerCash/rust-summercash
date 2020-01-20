use std::collections; // Import the stdlib collections library

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::{crypto::blake3, crypto::hash}; // Import the hash modules

use num::bigint::BigUint; // Add support for large unsigned integers

/// The state at a particular point in time.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    /// Body of the state entry
    pub data: EntryData,
    /// Hash of the state entry
    pub hash: hash::Hash,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntryData {
    /// Balances of every account at a certain point in time
    pub balances: collections::HashMap<String, BigUint>,
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
    pub fn new(balances: collections::HashMap<String, BigUint>) -> Entry {
        let entry_data: EntryData = EntryData {
            balances: balances, // Set balances
        }; // Initialize entry data

        let mut entry_data_bytes = vec![0; entry_data.to_bytes().len()]; // Initialize entry data buffer

        entry_data_bytes.clone_from_slice(entry_data.to_bytes().as_slice()); // Copy into buffer

        Entry {
            data: entry_data,                                      // Set data
            hash: blake3::hash_slice(entry_data_bytes.as_slice()), // Set hash
        }
    }
}

/// Merge multiple state entires into one batch state entry.
pub fn merge_entries(entries: Vec<Entry>) -> Entry {
    let mut balances: collections::HashMap<String, BigUint> = collections::HashMap::new(); // Initialize balances map

    for entry in entries {
        // Iterate through entries
        for (k, v) in entry.data.balances.iter() {
            // Iterate through balances
            if balances.contains_key(k) {
                // Check already exists in balances
                let balance_difference =
                    entry.data.balances.get(k).unwrap() - balances.get(k).unwrap(); // Calculate balance difference

                let mut_balance = balances.get_mut(k).unwrap(); // Get mutable balance

                *mut_balance += balance_difference; // Set balance to difference between balance at old state and balance at tx
            } else {
                balances.insert(k.to_string(), v.clone()); // Set balance
            }
        }
    }

    Entry::new(balances) // Return initialized state entry
}

#[cfg(test)]
mod tests {
    use super::*; // Import names from parent module

    use super::super::super::super::common::address; // Import the hash & address modules

    use crate::num::FromPrimitive; // Let the bigint library implement from_i64

    #[test]
    pub fn test_new() {
        let mut balances: collections::HashMap<String, BigUint> = collections::HashMap::new(); // Initialize balances hash map

        balances.insert(
            blake3::hash_slice(b"test").to_str(),
            BigUint::from_i64(1).unwrap(),
        ); // Balance of 1 fink

        let entry: Entry = Entry::new(balances); // Initialize state entry

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
