use jsonrpc_core::{response::Output, Error, ErrorCode, IoHandler, Result};
use jsonrpc_derive::rpc;

use serde::Deserialize;

use super::{
    super::super::{accounts::account::Account, common::address::Address},
    error,
};

use std::collections::HashMap;

/// Defines the standard SummerCash accounts RPC API.
#[rpc]
pub trait Accounts {
    /// Generates a new account and returns the account's address and private key. Note: this method also writes the
    /// new account to the given data directory.
    #[rpc(name = "new_account")]
    fn generate(&self, data_dir: String) -> Result<Account>;

    /// Reads an account with the given address from the disk, and returns its details. If the account is locked,
    /// an error will be returned.
    #[rpc(name = "get_account")]
    fn get(&self, address: Address, data_dir: String) -> Result<Account>;
}

/// An implementation of the accounts API.
pub struct AccountsImpl;

impl Accounts for AccountsImpl {
    /// Generates a new account and returns the account's address and private key
    fn generate(&self, data_dir: String) -> Result<Account> {
        // Generate an account
        let acc: Account = Account::new();

        // Persist the account to the local disk + return it
        match acc.write_to_disk_at_data_directory(&data_dir) {
            Ok(_) => Ok(acc),
            Err(_) => Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_WRITE_ACCOUNT,
            ))),
        }
    }

    /// Reads an account with the given address from the disk, and returns its details. If the account is locked,
    /// an error will be returned.
    fn get(&self, address: Address, data_dir: String) -> Result<Account> {
        // Convert the IO error into a suitable JSONRPC error, if need be
        match Account::read_from_disk_at_data_directory(address, &data_dir) {
            Ok(acc) => Ok(acc),
            Err(_) => Err(Error::new(ErrorCode::ServerError(
                error::ERROR_UNABLE_TO_OPEN_ACCOUNT,
            ))),
        }
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

    /// Performs a request considering the given method, and returns the response.
    async fn do_request<T>(
        &self,
        method: &str,
        params: &str,
    ) -> std::result::Result<T, failure::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Make a hashmap to store the body of the request in
        let mut json_body: HashMap<&str, serde_json::Value> = HashMap::new();
        json_body.insert("jsonrpc", serde_json::Value::String("2.0".to_owned()));
        json_body.insert("method", serde_json::Value::String(method.to_owned()));
        json_body.insert("id", serde_json::Value::String("".to_owned()));
        json_body.insert("params", serde_json::from_str(params)?);

        // Send a request to the endpoint, and pass the given parameters along with the request
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

    /// Generates and returns a new account.
    pub async fn generate(&self, data_dir: &str) -> std::result::Result<Account, failure::Error> {
        // Generate the account and return it
        self.do_request::<Account>("new_account", &format!("[\"{}\"]", data_dir))
            .await
    }

    /// Reads an account with the given address from the disk, and returns its details. If the account is locked,
    /// an error will be returned.
    pub async fn get(
        &self,
        address: Address,
        data_dir: &str,
    ) -> std::result::Result<Account, failure::Error> {
        self.do_request::<Account>(
            "get_account",
            &format!("[{}, \"{}\"]", serde_json::to_string(&address)?, data_dir,),
        )
        .await
    }
}
