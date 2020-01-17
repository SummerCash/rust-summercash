use super::super::super::common::address::Address;
use num::{BigUint, Zero};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};

/// The configuration for the network's genesis.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The capital allocated to each user
    pub(crate) alloc: HashMap<Address, BigUint>,

    /// The total value of the genesis
    total_value: BigUint,
}

impl Config {
    /// Initializes a new configuration.
    pub fn new() -> Config {
        // Just initialize an empty configuration, and return it
        Config {
            alloc: HashMap::new(),
            total_value: BigUint::zero(),
        }
    }

    /// Allocates the given amount to a particular address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address that capital will be allocated towards
    /// * `amount` - The amount of capital allocated to the address
    pub fn allocate_to_address(&mut self, address: Address, amount: BigUint) {
        // Put the amount in the alloc map
        self.alloc.insert(address, amount);
    }

    /// Identifies the amount of coins allocated to the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The desired address to which we will request their # of allocated coins
    pub fn amount_allocated_for_address(&self, address: Address) -> BigUint {
        // Return 0 if the address doesn't exist in the alloc map
        self.alloc.get(&address).unwrap_or(&BigUint::zero()).clone()
    }

    /// Identifies the amount of coins allocated in the genesis config.
    pub fn issuance(&self) -> BigUint {
        // Return the total value of the genesis config
        self.total_value.clone()
    }

    /// Reads a genesis configuration from the given genesis file in a given data dir.
    pub fn read_from_file(data_dir: &str, network: &str) -> Result<Self, failure::Error> {
        // Open the genesis configuration file
        let file = File::open(format!("{}/genesis/{}.json", data_dir, network))?;

        // Get a reader for the file
        let reader = BufReader::new(file);

        // Deserialize the config, and return it
        Ok(serde_json::from_reader(reader)?)
    }
}
