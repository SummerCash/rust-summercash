use std::collections; // Import the stdlib collections library

use serde::{Deserialize, Serialize}; // Import serde serialization

use num::bigint::BigUint; // Add support for large unsigned integers

use super::super::super::super::{common::address, crypto::blake2, crypto::hash}; // Import the hash & address modules

/// The state at a particular point in time.
pub struct Entry {
    /// Body of the state entry
    data: EntryData,
    /// Hash of the state entry
    hash: hash::Hash,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EntryData {
    /// Balances of every account at point in time
    balances: collections::HashMap<address::Address, BigUint>,
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
    /// Initialize a new Entry instance
    pub fn new(balances: collections::HashMap<address::Address, BigUint>) -> Entry {
        let entry_data: EntryData = EntryData{
            balances: balances, // Set balances
        }; // Initialize entry data

        let mut entry_data_bytes = vec![0; entry_data.to_bytes().len()]; // Initialize entry data buffer

        entry_data_bytes.clone_from_slice(entry_data.to_bytes().as_slice()); // Copy into buffer

        Entry{
            data: entry_data, // Set data
            hash: blake2::hash_slice(entry_data_bytes.as_slice()), // Set hash
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*; // Import names from parent module

//     #[test]
//     pub fn test_new() {
//         let entry: Entry = Entry::new()
//     }
// }