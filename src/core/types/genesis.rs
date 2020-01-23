use super::super::super::common::address::Address;
use num::{BigUint, FromPrimitive, Zero};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, default::Default, fs::File, io::BufReader};

/// The configuration for the network's genesis.
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    /// The capital allocated to each user
    pub(crate) alloc: HashMap<Address, BigUint>,

    /// The total value of the genesis
    total_value: BigUint,
}

impl Config {
    /// Allocates the given amount to a particular address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address that capital will be allocated towards
    /// * `amount` - The amount of capital allocated to the address
    pub fn allocate_to_address(&mut self, address: Address, amount: BigUint) {
        // Increment the total value of the allocation
        self.total_value += amount.clone();

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
    pub fn read_from_file(file_name: &str) -> Result<Self, failure::Error> {
        /// The raw configuration stored on disk, in JSON format with hex addresses, rather than inline vecs.
        #[derive(Deserialize)]
        struct RawConfig {
            alloc: HashMap<String, i128>,
        };

        // Open the genesis configuration file
        let file = File::open(file_name)?;

        // Get a reader for the file
        let reader = BufReader::new(file);

        // Deserialize the raw config. We still have some more work to do, though.
        let raw_cfg: RawConfig = serde_json::from_reader(reader)?;

        // We'll convert all of the strings into their address representations
        let mut final_cfg: Self = Self::default();

        // Go through each address, and its corresponding value. Put these values & addrs into the final configuration obj.
        for (address, value) in raw_cfg.alloc.iter() {
            // Put the key pair into the final configuration
            final_cfg.allocate_to_address(
                Address::from(address.as_ref()),
                BigUint::from_i128(*value).unwrap_or_default(),
            );
        }

        // Return the final configuration instance
        Ok(final_cfg)
    }
}
