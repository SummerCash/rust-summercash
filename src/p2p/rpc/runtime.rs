use jsonrpc_core::{response::Output, Error, ErrorCode, IoHandler, Result};
use jsonrpc_derive::rpc;

use serde::Deserialize;

use super::{
    super::super::core::sys::{proposal::Proposal, system::System},
    error,
};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[rpc]
pub trait Runtime {
    /// Gets a list of proposals contained in the runtime
    #[rpc(name = "list_pending_proposals")]
    fn list_pending_proposals(&self) -> Result<Vec<Proposal>>;
}

/// An implementation of the runtime API.
pub struct RuntimeImpl {
    pub(crate) runtime: Arc<RwLock<System>>,
}

impl Runtime for RuntimeImpl {
    /// Gets a list of proposals contained in the runtime
    fn list_pending_proposals(&self) -> Result<Vec<Proposal>> {
        // Get a list of the pending proposals contained in the runtime
        if let Ok(rt) = self.runtime.read() {
            Ok(rt.pending_proposals.values().cloned().collect())
        } else {
            Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_OBTAIN_LOCK,
            )))
        }
    }
}

impl RuntimeImpl {
    /// Registers the DAG service on the given IoHandler server.
    pub fn register(io: &mut IoHandler, runtime: Arc<RwLock<System>>) {
        // Register this service on the IO handler
        io.extend_with(Self { runtime }.to_delegate());
    }
}

/// A client for the runtime API.
pub struct Client {
    /// The address for the server hosting the API
    pub server: String,

    /// The HTTP client
    client: reqwest::Client,
}

impl Client {
    /// Initializes a new Client with the given remote URL.
    pub fn new(server_addr: &str) -> Self {
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

    /// Gets a list of proposals contained in the runtime
    pub async fn list_pending_proposals(
        &self,
    ) -> std::result::Result<Vec<Proposal>, failure::Error> {
        self.do_request::<Vec<Proposal>>("list_pending_proposals", "[]")
            .await
    }
}
