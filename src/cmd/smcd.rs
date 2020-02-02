/// SMCd is the SummerCash node daemon.
#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate tokio;
//extern crate tokio_01;

use failure::Error;
use libp2p::{Multiaddr, PeerId};
use summercash::{
    core::{sys::system::System, types::genesis::Config},
    p2p::{
        client::Client,
        network, peers,
        rpc::{accounts::AccountsImpl, dag::DagImpl},
    },
};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
};

use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;

/// The SummerCash node daemon.
#[derive(Clap)]
#[clap(version = "1.0", author = "Dowland A.")]
struct Opts {
    /// Print debug info
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Prevents any non-critical information from being printed to the console
    #[clap(short = "s", long = "silent")]
    silent: bool,

    /// Prevents the local node from connecting to any bootstrap peers.
    #[clap(short = "nb", long = "no-bootstrap")]
    no_bootstrap: bool,

    /// Disables the SummerCash RPC API, leaving the node in a completely isolated state.
    #[clap(short = "i", long = "isolated")]
    disable_api: bool,

    /// Signals to the local node that it should prefer the given port for all incoming operations.
    #[clap(short = "p", long = "node-port", default_value = "0")]
    node_port: u16,

    /// Signals to the local node that it should listen on the givn port for all RPC API communications.
    #[clap(short = "P", long = "rpc-port", default_value = "8080")]
    api_port: u16,

    /// Ensures that the node will connect to the given network
    #[clap(long = "network", default_value = "andromeda")]
    network: String,

    /// Changes the directory that node data will be stored in
    #[clap(long = "data-dir", default_value = "data")]
    data_dir: String,

    /// Uses a given genesis configuration file to construct a new genesis state for the network.
    #[clap(long = "genesis-file", default_value = "none")]
    genesis_file: String,

    /// Uses a bootstrap peer with the given ID to connect to the network.
    #[clap(long = "bootstrap-peer-id", default_value = "net_bps")]
    bootstrap_peer: String,

    /// Uses a bootstrap peer with the given multi-address to connect to the network.
    #[clap(long = "bootstrap-peer-addr", default_value = "net_bps")]
    bootstrap_peer_addr: String,
}

/// Starts the SMCd node daemon.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Get any flags issued by the user
    let opts: Opts = use_options(Opts::parse())?;

    // Use the options
    let (bootstrap_nodes, opts) = use_bootstrap_peers(&opts.network.clone(), opts)?;

    // Get a client for the network that the user specified
    let mut c = Client::new(opts.network.clone().into(), &opts.data_dir)?;

    // Convert the client into its string representation
    let c_str: String = (&c).into();

    // Log the initialized client, as well as the network name
    info!("Initiated network client ({}): \n{}", opts.network, c_str);

    // If the user wants to make a genesis state, let's do it.
    if opts.genesis_file != "none" {
        // Construct the genesis state
        use_genesis_file(&mut c, &opts.genesis_file, &opts.network)?;
    }

    // If the user wants to activate the SummerCash RPC API, let's do it.
    if !opts.disable_api {
        // Initialize a server for the RPC API
        let mut io = IoHandler::new();

        // Register the accounts API
        AccountsImpl::register(&mut io, c.runtime.clone());
        DagImpl::register(&mut io, c.runtime.clone());

        // Create an HTTP server for the RPC API
        let server = ServerBuilder::new(io)
            .start_http(
                &format!("0.0.0.0:{}", opts.api_port)
                    .parse()
                    .expect("Node was unable to parsed the given API port."),
            )
            .expect("Node was unable to create an HTTP server for the SummerCash RPC API.");

        info!(
            "Starting an HTTP server for the SummerCash RPC API on port {}",
            opts.api_port
        );

        // Start listening in a different thread
        thread::spawn(move || server.wait());
    }

    // A context buffer indicating the state of the server. Used to handle ^c.
    let server_ctx: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

    // Get a reference to the client's runtime so that we can stop all operations on ^c if necessary
    let runtime_ctx: Arc<RwLock<System>> = c.runtime.clone();
    let server_ctx_cl = server_ctx.clone();
    let persistence_dir = opts.data_dir.clone();

    ctrlc::set_handler(move || {
        // Get a runtime that we can actually close
        if let Ok(rt) = runtime_ctx.write() {
            // Save everything up
            rt.config
                .write_to_disk_at_data_directory(&persistence_dir)
                .expect("Error writing the runtime config to the disk");
            rt.ledger
                .write_to_disk()
                .expect("Error writing the ledger to the disk");
            server_ctx_cl.store(false, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Start the client
    c.start(server_ctx, bootstrap_nodes, opts.node_port).await?;

    // We're done!
    Ok(())
}

/// Gets the network bootstrap peers from the configuration struct.
fn use_bootstrap_peers(
    network: &str,
    opts: Opts,
) -> Result<(Vec<(PeerId, Multiaddr)>, Opts), Error> {
    // If the user has provided a custom bootstrap peer, use that.
    if opts.bootstrap_peer != "net_bps" && opts.bootstrap_peer_addr != "net_bps" {
        Ok((
            vec![(
                opts.bootstrap_peer.clone().parse()?,
                opts.bootstrap_peer_addr.clone().parse()?,
            )],
            opts,
        ))
    } else if opts.no_bootstrap {
        // The user has explicitly requested that they not connect to any bootstrap peers. Follow this wish.
        Ok((vec![], opts))
    } else {
        // Otherwise, just use the hard-coded bootstrap node for the active network
        Ok((
            peers::get_network_bootstrap_peers(network::Network::from(network)),
            opts,
        ))
    }
}

/// Constructs a new genesis for the network, considering a given genesis file.
fn use_genesis_file(client: &mut Client, file: &str, network: &str) -> Result<(), Error> {
    // Log the pending gen op
    info!(
        "Constructing a new network ({}) genesis state from the given file: {}",
        network, file
    );

    // Make the genesis state for the network
    client.construct_genesis(Config::read_from_file(file)?)?;

    // All done!
    Ok(())
}

/// Applies the given options.
fn use_options(mut opts: Opts) -> Result<Opts, Error> {
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
