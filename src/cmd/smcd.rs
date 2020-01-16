/// SMCd is the SummerCash node daemon.
#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate tokio;

use failure::Error;
use summercash::p2p::client::Client;

/// The SummerCash node daemon.
#[derive(Clap)]
struct Opts {
    /// Print debug info
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Prevents any non-critical information from being printed to the console
    #[clap(short = "s", long = "silent")]
    silent: bool,

    /// Ensures that the node will connect to the given network
    #[clap(long = "network", default_value = "andromeda")]
    network: String,
}

/// Starts the SMCd node daemon.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Get any flags issued by the user
    let opts: Opts = Opts::parse();

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

    // Get a client for the network that the user specified
    let c: Client = Client::new(opts.network.clone().into())?;

    // Convert the client into its string representation
    let c_str: String = (&c).into();

    // Log the initialized client, as well as the network name
    info!("Initiated network client ({}): \n{}", opts.network, c_str);

    // Start the client
    c.start().await?;

    // We're done!
    Ok(())
}
