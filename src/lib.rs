pub mod common; // Export the common module
pub mod core; // Export the core module
pub mod crypto; // Export the crypto module

#[macro_use]
extern crate failure; // Link failure crate

extern crate bincode; // Link bincode crate
extern crate blake2; // Link blake2 hashing library
extern crate chrono; // Link chrono library
extern crate ed25519_dalek; // Link edwards25519 library
extern crate hex; // Link hex encoding library
extern crate num; // Link num library
extern crate path_clean; // Link path clean crate
extern crate rand; // Link rand library
extern crate serde; // Link serde
extern crate serde_json; // Link serde
extern crate sled; // Link sled crate
