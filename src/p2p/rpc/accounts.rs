use jsonrpc_core::{response::Output, IoHandler, Result};
use jsonrpc_derive::rpc;

use super::super::super::accounts::account::Account;

use std::collections::HashMap;

/// Defines the standard SummerCash accounts RPC API.
#[rpc]
pub trait Accounts {
    /// Generates a new account and returns the account's address and private key
    #[rpc(name = "new_account")]
    fn generate(&self) -> Result<Account>;
}

/// An implementation of the accounts API.
pub struct AccountsImpl;

impl Accounts for AccountsImpl {
    /// Generates a new account and returns the account's address and private key
    fn generate(&self) -> Result<Account> {
        // Generate & return the account
        Ok(Account::new())
    }
}

impl AccountsImpl {
    /// Registers the accounts service on the given IoHandler server.
    pub fn register(io: &mut IoHandler) {
        // Register this service on the IO handler
        io.extend_with(Self.to_delegate());
    }
}

/// A client for the SummerCash accounts API.
pub struct Client {
    /// The address for the server hosting the APi
    pub server: String,

    /// An HTTP client
    client: reqwest::Client,
}

impl Client {
    /// Initializes a new Client with the given remote URL.
    pub fn new(server_addr: &str) -> Self {
        // Initialize and return the client
        Self {
            server: server_addr.trim_end_matches("/").to_owned(),
            client: reqwest::Client::new(),
        }
    }

    /// Generates and returns a new account.
    pub async fn generate(&self) -> std::result::Result<Account, failure::Error> {
        // Make a hashmap to store the body of the request in
        let mut json_body = HashMap::new();
        json_body.insert("jsonrpc", "2.0");
        json_body.insert("method", "new_account");
        json_body.insert("id", "");

        // Send a request to generate a new account to the server, and return the account
        let res = self
            .client
            .post(&self.server)
            .json(&json_body)
            .send()
            .await?
            .json::<Output>()
            .await?;

        // Some type conversion black magic fuckery
        match res {
            Output::Success(s) => match serde_json::from_value(s.result) {
                Ok(res) => Ok(res),
                Err(e) => Err(e.into()),
            },
            Output::Failure(e) => Err(e.error.into()),
        }
    }
}
