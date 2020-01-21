/// SMCli is the SummerCash command-line interface.
extern crate clap;
#[macro_use]
extern crate log;
use clap::Clap;

use summercash::{common::address::Address, crypto::hash::Hash, p2p::rpc::accounts};

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
}

#[derive(Clap, Clone)]
enum Create {
    /// Creates a new account.
    Account,
}

#[derive(Clap, Clone)]
enum Get {
    /// Gets a particular account with the given address.
    Account(Account),
}

#[derive(Clap, Clone)]
struct Account {
    /// The address of the account
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // Get the options that the user passed to the program
    let opts: Opts = use_options(Opts::parse())?;

    match opts.subcmd.clone() {
        SubCommand::Create(c) => create(opts, c).await,
        SubCommand::Get(c) => get(opts, c).await,
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
                Ok(acc) => info!("Successfully generated account: {}", acc),
                Err(e) => error!("Failed to generate account: {}", e),
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
                .get(Address::from(Hash::from_str(&acc.address)?), &opts.data_dir)
                .await
            {
                Ok(acc) => info!("Found account: {}", acc),
                Err(e) => error!("Failed to load the account: {}", e),
            }
        }
    };

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
