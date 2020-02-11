/// SMCli is the SummerCash command-line interface.
extern crate clap;
extern crate console;
#[macro_use]
extern crate log;
use clap::Clap;

use summercash::{
    cmd::commands::*,
    crypto::hash::Hash,
    p2p::rpc::{accounts, dag, runtime},
};

use console::Emoji;
use std::clone::Clone;

/// The SummerCash command-line interface.
#[derive(Clap)]
#[clap(version = "1.0", author = "Dowland A.")]
struct Opts {
    /// Print debug info
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Prevents any non-critical information from being printed to the console
    #[clap(short = "s", long = "silent")]
    silent: bool,

    /// Changes the directory that node data will be stored in
    #[clap(long = "data-dir", default_value = "data")]
    data_dir: String,

    /// Signals to the SummerCash command-line utility that it should connect to the given node.
    #[clap(
        short = "r",
        long = "remote-host-url",
        default_value = "http://127.0.0.1:8080"
    )]
    rpc_host_url: String,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Clone)]
enum SubCommand {
    /// Creates a new SummerCash object of a given type.
    #[clap(name = "create")]
    Create(Create),

    /// Gets a SummerCash object of a given type.
    #[clap(name = "get")]
    Get(Get),

    /// Locks a SummerCash object of a given type.
    #[clap(name = "lock")]
    Lock(Lock),

    /// Unlocks a SummerCash object of a given type.
    #[clap(name = "unlock")]
    Unlock(Unlock),

    /// Deletes a SummerCash object of a given type.
    #[clap(name = "delete")]
    Delete(Delete),

    /// Gets a list of SummerCash objects of a given type.
    #[clap(name = "list")]
    List(List),

    /// Signs a SummerCash object of a given type using a provided key.
    #[clap(name = "sign")]
    Sign(Sign),

    /// Publishes a SummerCash object of a given type using a provided hash.
    #[clap(name = "publish")]
    Publish(Publish),
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // Get the options that the user passed to the program
    let opts: Opts = use_options(Opts::parse())?;

    match opts.subcmd.clone() {
        SubCommand::Create(c) => create(opts, c).await,
        SubCommand::Get(c) => get(opts, c).await,
        SubCommand::Lock(l) => lock(opts, l).await,
        SubCommand::Unlock(u) => unlock(opts, u).await,
        SubCommand::Delete(d) => delete(opts, d).await,
        SubCommand::List(l) => list(opts, l).await,
        SubCommand::Sign(s) => sign(opts, s).await,
        SubCommand::Publish(p) => publish(opts, p).await,
    }
}

/// Creates the object from the given options.
async fn create(opts: Opts, c: Create) -> Result<(), failure::Error> {
    match c {
        Create::Account => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Generate the account
            match client.generate(&opts.data_dir).await {
                Ok(acc) => info!(
                    "{}Successfully generated account: {}",
                    Emoji::new("💳 ", ""),
                    acc
                ),
                Err(e) => error!("Failed to generate account: {}", e),
            }
        }
        Create::Transaction(transaction) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            // Generate the account
            match client
                .create_tx(
                    transaction.sender,
                    transaction.recipient,
                    transaction.amount,
                    transaction.payload,
                )
                .await
            {
                Ok(tx) => {
                    // Persist the tx first
                    tx.to_disk_at_data_directory(&opts.data_dir)?;

                    info!(
                        "{}Successfully created transaction (use publish command to add to DAG): {}", Emoji::new("📋 ", ""),
                        serde_json::to_string_pretty(&tx)?
                    );
                }
                Err(e) => error!("Failed to create transaction: {}", e),
            }
        }
    };

    Ok(())
}

/// Gets the object with matching criteria.
async fn get(opts: Opts, g: Get) -> Result<(), failure::Error> {
    match g {
        Get::Account(acc) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Get the account
            match client
                .get(Hash::from(acc.address.as_ref()), &opts.data_dir)
                .await
            {
                Ok(acc) => info!("{}Found account: {}", Emoji::new("📒 ", ""), acc),
                Err(e) => error!("Failed to load the account: {}", e),
            }
        }
        Get::Balance(acc) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Get the account
            match client.balance(Hash::from(acc.address.as_ref())).await {
                Ok(balance) => info!(
                    "{}Balance: {} SMC",
                    Emoji::new("💵 ", ""),
                    summercash::common::fink::convert_finks_to_smc(balance),
                ),
                Err(e) => error!("Failed to calculate the account's balance: {}", e),
            }
        }
        Get::Dag(_) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            match client.get().await {
                Ok(nodes) => {
                    info!("{}Loaded the DAG successfully!", Emoji::new("⛓️ ", ""));

                    if nodes.len() != 0 {
                        // Print out each of the nodes
                        for node in nodes {
                            println!("{}", serde_json::to_string_pretty(&node)?);
                        }
                    } else {
                        info!("{}It's empty tho...", Emoji::new("🤔 ", ""));
                    }
                }
                Err(e) => error!("Failed to load the DAG: {}", e),
            }
        }
        Get::TransactionMemory(_) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            match client.get_mem_txs(opts.data_dir).await {
                Ok(txs) => info!(
                    "{}",
                    txs.iter()
                        .fold(String::new(), |acc, &arg| acc + &arg.to_str())
                ),
                Err(e) => error!("Failed to load transactions stored in memory: {}", e),
            }
        }
    };

    Ok(())
}

/// Locks the object with matching constraints.
async fn lock(opts: Opts, l: Lock) -> Result<(), failure::Error> {
    match l {
        Lock::Account(acc) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Lock the account
            match client
                .lock(Hash::from(acc.address.as_ref()), &acc.key, &opts.data_dir)
                .await
            {
                Ok(_) => info!(
                    "{}Locked account '{}' successfully",
                    Emoji::new("🔒 ", ""),
                    acc.address
                ),
                Err(e) => error!("Failed to lock the account: {}", e),
            }
        }
    };

    Ok(())
}

/// Locks the object with matching constraints.
async fn unlock(opts: Opts, u: Unlock) -> Result<(), failure::Error> {
    match u {
        Unlock::Account(acc) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Lock the account
            match client
                .unlock(Hash::from(acc.address.as_ref()), &acc.key, &opts.data_dir)
                .await
            {
                Ok(acc) => info!(
                    "{}Unlocked account successfully: {}",
                    Emoji::new("🔓 ", ""),
                    acc
                ),
                Err(e) => error!("Failed to lock the account: {}", e),
            }
        }
    };

    Ok(())
}

/// Deletes the object with matching constraints.
async fn delete(opts: Opts, d: Delete) -> Result<(), failure::Error> {
    match d {
        Delete::Account(acc) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // Delete the account
            match client
                .delete(Hash::from(acc.address.as_ref()), &opts.data_dir)
                .await
            {
                Ok(_) => info!(
                    "{}Deleted account '{}' successfully",
                    Emoji::new("🗑️ ", ""),
                    acc.address
                ),
                Err(e) => error!("Failed to delete account '{}': {}", acc.address, e),
            }
        }
    };

    Ok(())
}

/// Lists the objects with the given type.
async fn list(opts: Opts, l: List) -> Result<(), failure::Error> {
    match l {
        List::Accounts(_) => {
            // Make a client for the accounts API
            let client = accounts::Client::new(&opts.rpc_host_url);

            // List all of the accounts on the disk
            match client.list(&opts.data_dir).await {
                // Print out each of the accounts' addresses
                Ok(accounts) => {
                    // The collective addresses of each of the accounts, in one string
                    let mut accounts_string = String::new();

                    // The current index in the addr collection process
                    let mut i = 0;

                    // Put each of the addresses into the overall string
                    let _: Vec<()> = accounts
                        .iter()
                        .map(|addr| {
                            // Append the address to the overall string (+ a separator, if need be)
                            accounts_string +=
                                &format!("{}{}", if i > 0 { ", " } else { "" }, addr.to_str());

                            // Increment the current index
                            i += 1;
                        })
                        .collect();

                    info!(
                        "{}Found accounts: {}",
                        Emoji::new("🔎 ", ""),
                        accounts_string
                    );
                }

                // Log the error
                Err(e) => error!("Failed to locate all of the accounts in dir: {}", e),
            }
        }
        List::Transactions(_) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            // List all of the transactions on the disk
            match client.list().await {
                Ok(transactions) => {
                    // The collective hashes of each transaction, in one string
                    let mut transactions_string = String::new();

                    // The current index in the hash collection process
                    let mut i = 0;

                    // Put each of the hashes into the overall string
                    let _: Vec<()> = transactions
                        .iter()
                        .map(|hash| {
                            // Append the hash to the overall string (+ a separator, if need be)
                            transactions_string +=
                                &format!("{}{}", if i > 0 { ", " } else { "" }, hash.to_str());

                            i += 1;
                        })
                        .collect();

                    info!(
                        "{}Found transactions: {}",
                        Emoji::new("🔎 ", ""),
                        transactions_string
                    );
                }

                // Log the error
                Err(e) => error!("Failed to locate all of the transactions in the DAG: {}", e),
            }
        }
        List::Proposals(_) => {
            // Make a client for the runtime API
            let client = runtime::Client::new(&opts.rpc_host_url);

            // List all of the pending proposals on the disk
            match client.list_pending_proposals().await {
                Ok(proposals) => {
                    let mut proposals_string = String::new();

                    // The current index in the proposals collection process
                    let mut i = 0;

                    let _: Vec<()> = proposals
                        .iter()
                        .map(|prop| {
                            proposals_string += &format!(
                                "{}Proposal to '{}' '{}': '{}' ({})",
                                if i > 1 { "\n" } else { "" },
                                prop.proposal_data.operation,
                                prop.proposal_data.param_name,
                                prop.proposal_name,
                                prop.proposal_id
                            );

                            i += 1;
                        })
                        .collect();

                    info!(
                        "{}Found proposals: {}",
                        Emoji::new("🔎 ", ""),
                        proposals_string
                    );
                }

                // Log the error
                Err(e) => error!(
                    "Failed to locate all of the proposals in the runtime: {}",
                    e
                ),
            }
        }
    }

    Ok(())
}

/// Signs the object with the provided keypair and options.
async fn sign(opts: Opts, s: Sign) -> Result<(), failure::Error> {
    match s {
        Sign::Transaction(signable) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            // Sign the transaction
            match client
                .sign_tx(signable.hash, signable.account, opts.data_dir)
                .await
            {
                Ok(signature) => info!(
                    "{}Signed tx (publish with publish command): {}",
                    Emoji::new("✍️ ", ""),
                    serde_json::to_string(&signature)?
                ),
                Err(e) => error!("Failed to sign tx: {}", e),
            }
        }
    }

    Ok(())
}

/// Publishes the object with the provided hash.
async fn publish(opts: Opts, p: Publish) -> Result<(), failure::Error> {
    match p {
        Publish::Transaction(publishable) => {
            // Make a client for the DAG API
            let client = dag::Client::new(&opts.rpc_host_url);

            match client.publish_tx(publishable.hash, opts.data_dir).await {
                Ok(_) => info!("{}Publishing transaction!", Emoji("🚚 ", "")),
                Err(e) => error!("Failed to publish tx: {}", e),
            }
        }
    }

    Ok(())
}

/// Applies the given options.
fn use_options(mut opts: Opts) -> Result<Opts, failure::Error> {
    // Configure the logger
    if !opts.silent {
        if opts.debug {
            // Include debug statements in the logger output
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .init();
        } else {
            // Include just up to info statements
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .init();
        }
    }

    // If the user has chosen the default data dir, normalize it
    if opts.data_dir == "data" {
        // Normalize the data directory, and put it back in the config
        opts.data_dir = summercash::common::io::data_dir();
    }

    Ok(opts)
}
