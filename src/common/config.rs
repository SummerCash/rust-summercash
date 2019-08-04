use num::bigint::BigUint; // Add support for large unsigned integers

use std::fs; // Import the filesystem library
use std::io; // Import the io library

use serde::{Deserialize, Serialize}; // Import serde serialization

/// The current version of rust-summercash.
pub static NODE_VERSION: &str = "v0.1.0";

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
        let mut file = fs::File::create(super::common::io::format_config_dir(format!("network_{}.config", self.network_name)))?; // Initialize file
        file.write_all(b"Hello, world!")?;
        Ok(())
    }
}