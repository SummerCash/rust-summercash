use jsonrpc_core::{response::Output, Error, ErrorCode, IoHandler, Result};
use jsonrpc_derive::rpc;

use serde::Deserialize;

use super::{
    super::super::{
        core::{sys::system::System, types::{graph::Node, transaction::Transaction}},
        crypto::hash::Hash,
    },
    error,
};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Defines the standard SummerCash DAG RPC API.
#[rpc]
pub trait Dag {
    /// Gets a list of nodes contained in the currently attached network's DAG.
    #[rpc(name = "get_dag")]
    fn get(&self) -> Result<Vec<Node>>;

    /// Gets a list of transaction hashes stored in the currently attached DAG.
    #[rpc(name = "list_transactions")]
    fn list(&self) -> Result<Vec<Hash>>;

    /// Creates a new transaction with the provided sender, recipient, value, and payload.
    fn create_tx(&mut self, sender: String, recipient: String, value: u64, payload: String) -> Result<Transaction>;
}

/// An implementation of the DAG API.
pub struct DagImpl {
    pub(crate) runtime: Arc<RwLock<System>>,
}

impl Dag for DagImpl {
    /// Gets a list of nodes contained in the currently attached network's DAG.
    fn get(&self) -> Result<Vec<Node>> {
        if let Ok(rt) = self.runtime.read() {
            // Return all of the nodes in the runtime's ledger
            Ok(rt.ledger.nodes.clone())
        } else {
            // Return the corresponding error
            Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_OBTAIN_LOCK,
            )))
        }
    }

    /// Gets a list of transaction hashes stored in the currently attached DAG.
    fn list(&self) -> Result<Vec<Hash>> {
        if let Ok(rt) = self.runtime.read() {
            // Return all of the keys, which are the node hashes, stored in the DAG
            Ok(rt.ledger.hash_routes.keys().map(|hash| *hash).collect())
        } else {
            // Return the corresponding error
            Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_OBTAIN_LOCK,
            )))
        }
    }

    /// Creates a new transaction with the provided sender, recipient, value, and payload.
    fn create_tx(&mut self, sender: String, recipient: String, value: u64, payload: String) -> Result<Transaction> {
        let transaction = Transaction::new(e) 
    }
}

impl DagImpl {
    /// Registers the DAG service on the given IoHandler server.
    pub fn register(io: &mut IoHandler, runtime: Arc<RwLock<System>>) {
        // Register this service on the IO handler
        io.extend_with(Self { runtime }.to_delegate());
    }
}

/// A client for the SummerCash DAG API.
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
            server: server_addr.trim_end_matches('/').to_owned(),
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

    /// Gets a list of nodes in the working graph.
    pub async fn get(&self) -> std::result::Result<Vec<Node>, failure::Error> {
        self.do_request::<Vec<Node>>("get_dag", "[]").await
    }

    /// Gets a list of transaction hashes contained in the working DAG.
    pub async fn list(&self) -> std::result::Result<Vec<Hash>, failure::Error> {
        self.do_request::<Vec<Hash>>("list_transactions", "[]")
            .await
    }
}
