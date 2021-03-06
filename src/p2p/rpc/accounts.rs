use jsonrpc_core::{response::Output, Error, ErrorCode, IoHandler, Result};
use jsonrpc_derive::rpc;

use cryptolib::{
    aes::{self, KeySize},
    blockmodes,
    buffer::{self, BufferResult, ReadBuffer, WriteBuffer},
};

use serde::Deserialize;

use rand::{rngs::StdRng, RngCore, SeedableRng};

use super::{
    super::super::{
        accounts::account::{self, Account},
        common::address::Address,
        core::sys::system::System,
        crypto::blake3,
    },
    error,
};

use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, RwLock},
};

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

    /// Locks the account with the corresponding address in the given data directory. If the account is already locked,
    /// an error is returned.
    #[rpc(name = "lock_account")]
    fn lock(&self, address: Address, enc_key: String, data_dir: String) -> Result<()>;

    /// Unlocks the account with the corresponding address in the given data directory, and returns the account if the operation
    /// was successful. If the account is already unlocked, an error is returned.
    #[rpc(name = "unlock_account")]
    fn unlock(&self, address: Address, dec_key: String, data_dir: String) -> Result<Account>;

    /// Deletes the account with the corresponding address.
    #[rpc(name = "delete_account")]
    fn delete(&self, address: Address, data_dir: String) -> Result<()>;

    /// Gets a list of unlocked accounts on the disk.
    #[rpc(name = "list_accounts")]
    fn list(&self, data_dir: String) -> Result<Vec<Address>>;

    /// Gets the balance of an account with the given address.
    #[rpc(name = "get_account_balance")]
    fn balance(&self, address: Address) -> Result<num::BigUint>;
}

/// An implementation of the accounts API.
pub struct AccountsImpl {
    pub(crate) runtime: Arc<RwLock<System>>,
}

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

    /// Locks the account with the corresponding address in the given data directory. If the account is already locked,
    /// an error is returned.
    fn lock(&self, address: Address, enc_key: String, data_dir: String) -> Result<()> {
        // We'll need to generate an IV to do encryption properly
        let mut iv: [u8; 16] = [0; 16];

        // Make a random number generator for the pwd
        let mut rng: StdRng = SeedableRng::from_seed(*blake3::hash_slice(enc_key.as_bytes()));

        // Generate an IV
        rng.fill_bytes(&mut iv);

        // Make an instance of the encryption helper for the file
        let mut enc = aes::cbc_encryptor(
            KeySize::KeySize128,
            &*blake3::hash_slice(enc_key.as_bytes()),
            &iv,
            blockmodes::PkcsPadding,
        );

        // Open the file that the account is stored in
        let mut f = if let Ok(f) = fs::OpenOptions::new().read(true).write(true).open(format!(
            "{}/keystore/{}.json",
            data_dir,
            address.to_str()
        )) {
            f
        } else {
            // Return an error
            return Err(Error::new(ErrorCode::ServerError(
                error::ERROR_UNABLE_TO_OPEN_ACCOUNT,
            )));
        };

        // The contents of the file. We'll read the file into this buffer later.
        let mut contents = Vec::new();

        // Read into the buffer from the file.
        match f.read_to_end(&mut contents) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::new(ErrorCode::ServerError(
                    error::ERROR_UNABLE_TO_OPEN_ACCOUNT,
                )));
            }
        }

        // Idk some crypto stuff
        let mut read_buffer = buffer::RefReadBuffer::new(&contents);

        // Make a buffer to store the final encrypted data in
        let mut final_result = Vec::<u8>::new();

        // More crypto garbage
        let mut buffer = [0; 2048];

        // Even more crypto stuff
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        // Encrypt the stuff...
        loop {
            // Depersonalization isn't fun, but encryption is! Encrypt! Encrypt! Encrypt!
            let result = if let Ok(res) = enc.encrypt(&mut read_buffer, &mut write_buffer, true) {
                // We successfully encrypted the block. Cool.
                res
            } else {
                // Return an error!
                return Err(Error::new(ErrorCode::from(error::ERROR_ENCRYPTION_FAILED)));
            };

            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );

            // Handle some bad stuff...
            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }

        // Reset the writer to the beginning of the file
        if f.seek(SeekFrom::Start(0)).is_err() {
            // Return an error
            return Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_WRITE_ACCOUNT,
            )));
        }

        // Put the encrypted data in the file
        match f.write_all(&final_result) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_WRITE_ACCOUNT,
            ))),
        }
    }

    /// Unlocks the account with the corresponding address in the given data directory. If the account is already unlocked,
    /// an error is returned.
    fn unlock(&self, address: Address, dec_key: String, data_dir: String) -> Result<Account> {
        // We'll need to generate an IV to do encryption properly
        let mut iv: [u8; 16] = [0; 16];

        // Make a random number generator for the pwd
        let mut rng: StdRng = SeedableRng::from_seed(*blake3::hash_slice(dec_key.as_bytes()));

        // Generate an IV
        rng.fill_bytes(&mut iv);

        // Make an instance of the decryption helper for the file
        let mut dec = aes::cbc_decryptor(
            KeySize::KeySize128,
            &*blake3::hash_slice(dec_key.as_bytes()),
            &iv,
            blockmodes::PkcsPadding,
        );

        // Open the file that the account is stored in
        let mut f = if let Ok(f) = fs::OpenOptions::new().read(true).write(true).open(format!(
            "{}/keystore/{}.json",
            data_dir,
            address.to_str()
        )) {
            f
        } else {
            // Return an error
            return Err(Error::new(ErrorCode::ServerError(
                error::ERROR_UNABLE_TO_OPEN_ACCOUNT,
            )));
        };

        // The contents of the file. We'll have to decrypt this in a second.
        let mut contents = Vec::new();

        // Read into the buffer from the file.
        match f.read_to_end(&mut contents) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::new(ErrorCode::ServerError(
                    error::ERROR_UNABLE_TO_OPEN_ACCOUNT,
                )));
            }
        }

        // Generate a few buffers, set the encoder to read from the file's contents
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(&contents);
        let mut buffer = [0; 2048];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            // Try to decrypt the data in the file; return an error if this fails
            let result = if let Ok(res) = dec.decrypt(&mut read_buffer, &mut write_buffer, true) {
                res
            } else {
                // Return an error
                return Err(Error::new(ErrorCode::from(error::ERROR_DECRYPTION_FAILED)));
            };

            // Put the decrypted data block into the overall buffer
            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );

            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }

        // Deserialize the account
        match serde_json::from_slice(final_result.as_slice()) {
            Ok(acc) => {
                // Re-open the file, but with permissions that delete what was previously in the file
                f = if let Ok(opened_file) = fs::OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .open(format!("{}/keystore/{}.json", data_dir, address.to_str()))
                {
                    opened_file
                } else {
                    // Return an error
                    return Err(Error::new(ErrorCode::from(
                        error::ERROR_UNABLE_TO_WRITE_ACCOUNT,
                    )));
                };

                // Now that we've deserialized the account, let's write it back to the original file
                match serde_json::to_writer(f, &acc) {
                    Ok(_) => Ok(acc),
                    Err(_) => Err(Error::new(ErrorCode::from(
                        error::ERROR_UNABLE_TO_WRITE_ACCOUNT,
                    ))),
                }
            }
            Err(_) => Err(Error::new(ErrorCode::ServerError(
                error::ERROR_UNABLE_TO_READ_ACCOUNT,
            ))),
        }
    }

    /// Deletes the account with the corresponding address in the given data directory.
    fn delete(&self, address: Address, data_dir: String) -> Result<()> {
        // Delete the account
        match fs::remove_file(format!("{}/keystore/{}.json", data_dir, address)) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_DELETE_ACCOUNT,
            ))),
        }
    }

    /// Gets a list of unlocked accounts on the disk.
    fn list(&self, data_dir: String) -> Result<Vec<Address>> {
        // Return a list of unlocked accounts
        Ok(account::get_all_unlocked_accounts_in_data_directory(
            &data_dir,
        ))
    }

    /// Gets the balance of the account.
    fn balance(&self, address: Address) -> Result<num::BigUint> {
        // Get a runtime that we can use to get the balances at the last transaction
        let rt = if let Ok(runtime) = self.runtime.read() {
            runtime
        } else {
            // Return an error communicating the inability to obtain a read lock
            return Err(Error::new(ErrorCode::from(
                error::ERROR_UNABLE_TO_OBTAIN_LOCK,
            )));
        };

        // If there are no nodes, return a zero balance
        if rt.ledger.nodes.is_empty() {
            // Return a zero balance
            Ok(num::BigUint::default())
        } else {
            // The head of the graph, where its state has been executed properly
            let working_state = if let Some(n) = rt.ledger.obtain_executed_head() {
                n
            } else {
                // Return an error communicating the inability to read the node
                return Err(Error::new(ErrorCode::from(
                    error::ERROR_UNABLE_TO_OBTAIN_STATE_REF,
                )));
            };

            // Try to get the balance of the user from the state entry data, and return it
            if let Some(state) = &working_state.state_entry {
                Ok(state
                    .data
                    .balances
                    .get(&address.to_str())
                    .unwrap_or(&num::BigUint::default())
                    .clone())
            } else {
                // Return an error
                Err(Error::new(ErrorCode::from(
                    error::ERROR_UNABLE_TO_OBTAIN_STATE_REF,
                )))
            }
        }
    }
}

impl AccountsImpl {
    /// Registers the accounts service on the given IoHandler server.
    pub fn register(io: &mut IoHandler, runtime: Arc<RwLock<System>>) {
        // Register this service on the IO handler
        io.extend_with(Self { runtime }.to_delegate());
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
            &format!("[{}, \"{}\"]", serde_json::to_string(&address)?, data_dir),
        )
        .await
    }

    /// Encrypts an account with the giiven address. If the account is already locked, an error is returned.
    pub async fn lock(
        &self,
        address: Address,
        enc_key: &str,
        data_dir: &str,
    ) -> std::result::Result<(), failure::Error> {
        self.do_request::<()>(
            "lock_account",
            &format!(
                "[{}, \"{}\", \"{}\"]",
                serde_json::to_string(&address)?,
                enc_key,
                data_dir
            ),
        )
        .await
    }

    /// Decrypts an account with the given address. If the account is already unlocked, an error is returned.
    pub async fn unlock(
        &self,
        address: Address,
        dec_key: &str,
        data_dir: &str,
    ) -> std::result::Result<Account, failure::Error> {
        self.do_request::<Account>(
            "unlock_account",
            &format!(
                "[{}, \"{}\", \"{}\"]",
                serde_json::to_string(&address)?,
                dec_key,
                data_dir
            ),
        )
        .await
    }

    /// Deletes an account with the given address.
    pub async fn delete(
        &self,
        address: Address,
        data_dir: &str,
    ) -> std::result::Result<(), failure::Error> {
        self.do_request::<()>(
            "delete_account",
            &format!("[{}, \"{}\"]", serde_json::to_string(&address)?, data_dir),
        )
        .await
    }

    /// Gets a list of accounts stored on the disk in a given directory.
    pub async fn list(&self, data_dir: &str) -> std::result::Result<Vec<Address>, failure::Error> {
        self.do_request::<Vec<Address>>("list_accounts", &format!("[\"{}\"]", data_dir))
            .await
    }

    /// Gets the balance of a particular account.
    pub async fn balance(
        &self,
        address: Address,
    ) -> std::result::Result<num::BigUint, failure::Error> {
        self.do_request::<num::BigUint>(
            "get_account_balance",
            &format!("[{}]", serde_json::to_string(&address)?),
        )
        .await
    }
}
