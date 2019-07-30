pub mod common; // Export the common module
pub mod core;
pub mod crypto; // Export the crypto module // Export the core module

extern crate blake2; // Link blake2 hashing library
extern crate chrono; // Link chrono library
extern crate ed25519_dalek; // Link edwards25519 library
extern crate hex; // Link hex encoding library
extern crate num; // Link num library
extern crate serde;
extern crate time; // Link time library // Link serialization library
