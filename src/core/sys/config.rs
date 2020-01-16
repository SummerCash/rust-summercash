use num::bigint::BigUint; // Add support for large unsigned integers

use std::{fs, io, io::Write}; // Import the filesystem library

use serde::{Deserialize, Serialize}; // Import serde serialization

use super::super::super::common; // Import the io module

/// The current version of rust-summercash.
pub const NODE_VERSION: &str = "v0.1.0";

pub const DEFAULT_REWARD_PER_GAS: BigUint = BigUint::from(1000000 as u32);

/// A container specifying a set of SummerCash protocol constants.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// The amount of finks per gas to give as a reward for validating a tx (i.e. increase rewards across the board) TODO: Gas table
    pub reward_per_gas: BigUint,

    /// The name of the network
    pub network_name: String,
}

/// Implement a set of config helper methods.
impl Config {
    /// Persist a given config to the disk.
    pub fn write_to_disk(&self) -> io::Result<()> {
        fs::create_dir_all(common::io::config_dir())?; // Make config directory

        let mut file = fs::File::create(common::io::format_config_dir(&format!(
            "network_{}.json",
            self.network_name
        )))?; // Initialize file

        file.write_all(serde_json::to_vec_pretty(self)?.as_slice())?; // Serialize

        Ok(()) // All good!
    }

    /// Read a persisted config form the disk.
    pub fn read_from_disk(network_name: &str) -> io::Result<Config> {
        let file = fs::File::open(common::io::format_config_dir(&format!(
            "network_{}.json",
            network_name
        )))?; // Open config file

        Ok(serde_json::from_reader(file)?) // Return read config
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import names from parent module

    use std::str::FromStr; // Allow overriding of from_str() helper method.

    #[test]
    fn test_write_to_disk() {
        let config = Config {
            reward_per_gas: BigUint::from_str("10000000000000000000000000000000000000000").unwrap(), // Venezuela style
            network_name: "olympia".to_owned(),
        }; // Initialize network config

        config.write_to_disk().unwrap(); // Panic if not Ok()
    }

    #[test]
    fn test_read_from_disk() {
        let config = Config {
            reward_per_gas: BigUint::from_str("10000000000000000000000000000000000000000").unwrap(), // Venezuela style
            network_name: "olympia".to_owned(),
        }; // Initialize network config

        config.write_to_disk().unwrap(); // Panic if not Ok()

        let read_config = Config::read_from_disk("olympia").unwrap(); // Read config
        assert_eq!(read_config.reward_per_gas, config.reward_per_gas); // Ensure deserialized correctly
        assert_eq!(read_config.network_name, config.network_name); // Ensure deserialized correctly
    }
}
