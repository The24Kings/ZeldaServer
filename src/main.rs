use clap::Parser;
use std::sync::{Arc, Mutex, mpsc};
use std::{env, fs::File, net::TcpListener};
use tracing::{debug, info, warn};

use crate::logic::{commands::input, config::Config, map};
use crate::threads::{connection::connection, server::server};

pub mod logic;
pub mod threads;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to bind the TCP Connection
    #[arg(short, long, default_value_t = 5051)]
    port: u16,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() -> ! {
    let args = Args::parse();

    dotenvy::dotenv().expect("[MAIN] Failed to load .env file");
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .with_target(false)
        .with_ansi(true)
        .compact()
        .init();

    let server_config = Arc::new(Config::load());
    let client_config = server_config.clone(); // The Arc will handle all reference counting, it's not actually cloning all the data :)

    let address = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&address).expect("[MAIN] Failed to bind to address");

    info!("[MAIN] Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(rx));
    let sender = tx.clone();

    // Build the game map
    let path = env::var("MAP_FILEPATH").expect("[MAIN] MAP_FILEPATH must be set.");
    let file = File::open(path).expect("[MAIN] Failed to open map file!");
    let mut rooms = map::build(file).expect("[MAIN] Failed to build map from file");

    // Start the server thread with the map
    info!("[MAIN] Parsed map successfully");

    let _ = std::thread::spawn(move || {
        info!("[MAIN] Started server thread!");
        server(receiver, server_config, &mut rooms);
    });

    let _ = std::thread::spawn(move || {
        info!("[MAIN] Started input thread!");
        input(tx.clone());
    });

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("[MAIN] New connection: {}", addr);

                let stream = Arc::new(stream);
                let sender = sender.clone();
                let client_config = client_config.clone();

                // Handle the connection in a separate thread
                info!("[MAIN] Started connection thread!");

                let client_h = std::thread::spawn(move || {
                    connection(stream, sender, client_config);
                });

                debug!("[MAIN] Spawned client thread: {:?}", client_h.thread().id());
            }
            Err(e) => {
                warn!("[MAIN] Error accepting connection: {}", e);
            }
        }
    }
}
