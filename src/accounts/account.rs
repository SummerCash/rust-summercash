use libp2p::identity::{ed25519::Keypair, error}; // Import the libp2p library

use ed25519_dalek; // Import the edwards25519 digital signature library
use rand::rngs::OsRng; // Import the os's rng

use std::{fs, io, io::Write}; // Import the io library

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::{common, common::address}; // Import the address module

/// A SummerCash account.
#[derive(Serialize, Deserialize)]
pub struct Account {
    /// The account's private and public keys
    pub keypair: ed25519_dalek::Keypair,
    /// The account's p2p identity
    p2p_keypair: Vec<u8>,
}

/// Implement a set of account helper methods.
impl Account {
    /// Initialize a new account from a generated keypair.
    pub fn new() -> Account {
        let mut csprng: OsRng = OsRng::new().unwrap(); // Generate source of randomness

        Account {
            keypair: ed25519_dalek::Keypair::generate(&mut csprng), // Generate keypair
            p2p_keypair: Keypair::generate().encode().to_vec(),     // Generate p2p keypair
        } // Return account
    }

    /// Get the address of a particular account.
    pub fn address(&self) -> address::Address {
        address::Address::from_public_key(&self.keypair.public) // Return address
    }

    /// Get the p2p keypair of a particular account.
    pub fn p2p_keypair(&self) -> Result<Keypair, error::DecodingError> {
        Keypair::decode(self.p2p_keypair.clone().as_mut_slice()) // Return decoded keypair
    }

    /// Persist the account to the disk.
    pub fn write_to_disk(&self) -> io::Result<()> {
        fs::create_dir_all(common::io::keystore_dir())?; // Make keystore directory

        let mut file = fs::File::create(common::io::format_keystore_dir(&format!(
            "{}.json",
            self.address().to_str()
        )))?; // Initialize file
        file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?; // Serialize
        Ok(()) // All good!
    }

    /// Read an account from the disk.
    pub fn read_from_disk(address: address::Address) -> io::Result<Account> {
        let file = fs::File::open(common::io::format_keystore_dir(&format!(
            "{}.json",
            address.to_str()
        )))?; // Open account file

        Ok(serde_json::from_reader(file)?) // Return read account
    }
}
