use libp2p::identity::{ed25519::Keypair, error}; // Import the libp2p library

use ed25519_dalek; // Import the edwards25519 digital signature library
use rand::rngs::OsRng; // Import the os's rng

use walkdir::WalkDir; // Import the walkdir utility

use std::{fmt, fs, io, io::Write}; // Import the io library

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::{common, common::address, crypto::blake3}; // Import the address module

/// A SummerCash account.
#[derive(Serialize, Deserialize)]
pub struct Account {
    /// The account's private and public keys
    keypair: Vec<u8>,
    /// The account's p2p identity
    p2p_keypair: Vec<u8>,
}

/// Implement a set of account helper methods.
impl Account {
    /// Initialize a new account from a generated keypair.
    pub fn new() -> Account {
        let mut csprng = OsRng {};

        Account {
            keypair: ed25519_dalek::Keypair::generate(&mut csprng)
                .to_bytes()
                .to_vec(), // Generate keypair
            p2p_keypair: Keypair::generate().encode().to_vec(), // Generate p2p keypair
        } // Return account
    }

    /// Get the address of a particular account.
    pub fn address(&self) -> Result<address::Address, ed25519_dalek::SignatureError> {
        Ok(address::Address::from_public_key(&self.keypair()?.public)) // Return address
    }

    pub fn keypair(&self) -> Result<ed25519_dalek::Keypair, ed25519_dalek::SignatureError> {
        ed25519_dalek::Keypair::from_bytes(self.keypair.as_slice()) // Return decoded keypair
    }

    /// Get the p2p keypair of a particular account.
    pub fn p2p_keypair(&self) -> Result<Keypair, error::DecodingError> {
        Keypair::decode(self.p2p_keypair.clone().as_mut_slice()) // Return decoded keypair
    }

    /// Persist the account to the disk.
    pub fn write_to_disk(&self) -> io::Result<()> {
        fs::create_dir_all(common::io::keystore_dir())?; // Make keystore directory

        // Check could get address
        if let Ok(address) = self.address() {
            let mut file = fs::File::create(common::io::format_keystore_dir(&format!(
                "{}.json",
                address.to_str()
            )))?; // Initialize file
            file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?; // Serialize
            Ok(()) // All good!
        } else {
            Err(io::Error::from(io::ErrorKind::InvalidData)) // Return error
        }
    }

    /// Persist the account to the disk at a given data directory.
    pub fn write_to_disk_at_data_directory(&self, data_dir: &str) -> io::Result<()> {
        // Make a keystore directory in the data dir
        fs::create_dir_all(&format!("{}/keystore", data_dir))?;

        // Try to derive an address from the account's keypair
        if let Ok(address) = self.address() {
            // Initialize a file that we can save the account to
            let mut file =
                fs::File::create(&format!("{}/keystore/{}.json", data_dir, address.to_str()))?;
            // Serializ the account + write it to the file
            file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?;

            // Everything's good!
            Ok(())
        } else {
            // Return an error
            Err(io::Error::from(io::ErrorKind::InvalidData))
        }
    }

    /// Write an account to the disk at a given data directory.
    pub fn write_to_disk_with_name_at_data_directory(
        &self,
        s: &str,
        data_dir: &str,
    ) -> io::Result<()> {
        fs::create_dir_all(&format!("{}/keystore", data_dir))?; // Make a keystore directory in the data dir

        // Initialize a file to store the new account in
        let mut file = fs::File::create(&format!(
            "{}/keystore/{}.json",
            data_dir,
            blake3::hash_slice(s.as_bytes()).to_str()
        ))?;

        file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?; // Serialize the account, and put it in the new file

        Ok(()) // All good!
    }

    pub fn write_to_disk_with_name(&self, s: &str) -> io::Result<()> {
        fs::create_dir_all(common::io::keystore_dir())?; // Make keystore directory

        let mut file = fs::File::create(common::io::format_keystore_dir(&format!(
            "{}.json",
            blake3::hash_slice(s.as_bytes()).to_str()
        )))?; // Initialize file

        file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?; // Serialize

        Ok(()) // All good!
    }

    /// Read an account from the disk at a given data directory.
    pub fn read_from_disk_at_data_directory(
        address: address::Address,
        data_dir: &str,
    ) -> io::Result<Account> {
        // Open the file holding the account details corresponding to the address
        let file = fs::File::open(&format!("{}/keystore/{}.json", data_dir, address.to_str()))?;

        Ok(serde_json::from_reader(file)?) // Deserialize the account from the information held in the file + return
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

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Serialize)]
        struct HexRepresentation {
            private_key: String,
            address: String,
        }

        // Convert the account into a string struct
        let acc_hex = HexRepresentation {
            private_key: {
                if let Ok(kp) = self.keypair() {
                    hex::encode(kp.secret)
                } else {
                    "invalid_key".to_owned()
                }
            },
            address: {
                if let Ok(addr) = self.address() {
                    hex::encode(addr)
                } else {
                    "invalid_key".to_owned()
                }
            },
        };

        write!(
            f,
            "{}",
            serde_json::to_string_pretty(&acc_hex).unwrap_or_default()
        )
    }
}

/// Get a list of unlocked, localized accounts.
pub fn get_all_unlocked_accounts() -> Vec<Account> {
    let mut accounts: Vec<Account> = vec![]; // Initialize empty account addresses vec

    // Walk keystore directory
    for e in WalkDir::new(common::io::keystore_dir())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Ok(metadata) = e.metadata() {
            // Check is file
            if metadata.is_file() {
                // Convert path to string
                if let Some(path_str) = e.path().to_str() {
                    // Open account file
                    if let Ok(file) = fs::File::open(path_str) {
                        // Read account from file
                        if let Ok(account) = serde_json::from_reader(file) {
                            accounts.push(account); // Add account to account addresses vec
                        }
                    }
                }
            }
        }
    }

    accounts // Return accounts
}

#[cfg(test)]
mod tests {
    use super::*; // Import names from parent module

    #[test]
    fn test_new_peer_identity() {
        let peer_identity = Account::new(); // Generate a new account
        peer_identity
            .write_to_disk_with_name("peer_identity")
            .unwrap(); // Write peer identity to the disk
    }

    #[test]
    fn test_get_all_accounts() {
        let test_account = Account::new(); // Generate a new account
        test_account.write_to_disk().unwrap(); // Write test account to disk

        assert_ne!(get_all_unlocked_accounts().len(), 0); // Ensure has local accounts
    }

    #[test]
    fn test_read_from_disk() {
        let test_account = Account::new(); // Generate a anew account
        test_account.write_to_disk().unwrap(); // Write test account to disk

        let read_account = Account::read_from_disk(test_account.address().unwrap()).unwrap(); // Read account from disk

        assert_eq!(test_account.address(), read_account.address()); // Ensure accounts have same address
    }
}
