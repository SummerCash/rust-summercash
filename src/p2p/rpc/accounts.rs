use jsonrpc_core::{Error, ErrorCode, IoHandler, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

use ed25519_dalek::SecretKey;

use super::{
    super::super::{accounts::account::Account, common::address::Address},
    error,
};

/// Represents a response from the NewAccount method.
#[derive(Serialize, Deserialize)]
pub struct NewAccount {
    /// The address of the account
    pub address: Address,

    /// The private key of the account
    pub private_key: SecretKey,
}

/// Defines the standard SummerCash accounts RPC API.
#[rpc]
pub trait Accounts {
    /// Generates a new account and returns the account's address and private key
    #[rpc(name = "new")]
    fn generate(&self) -> Result<NewAccount>;
}

/// An implementation of the accounts API.
pub struct AccountsImpl;

impl Accounts for AccountsImpl {
    /// Generates a new account and returns the account's address and private key
    fn generate(&self) -> Result<NewAccount> {
        // Generate the account
        let acc = Account::new();

        let address = if let Ok(addr) = acc.address() {
            addr
        } else {
            // Return a server error, since we weren't able to derived the account's address
            return Err(Error::new(ErrorCode::ServerError(
                error::ERROR_SIGNATURE_UNDEFINED,
            )));
        };

        // Get the account's keypair, return an error if this failed
        let private_key = if let Ok(kp) = acc.keypair() {
            kp.secret
        } else {
            // Return a server error, since we should have been able to get a keypair for this account
            return Err(Error::new(ErrorCode::ServerError(
                error::ERROR_SIGNATURE_UNDEFINED,
            )));
        };

        Ok(NewAccount {
            address,
            private_key,
        })
    }
}

impl AccountsImpl {
    /// Registers the accounts service on the given IoHandler server.
    pub fn register(io: &mut IoHandler) {
        // Register this service on the IO handler
        io.extend_with(Self.to_delegate());
    }
}
