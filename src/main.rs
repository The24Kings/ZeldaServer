use clap::Parser;
use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::sync::{Arc, Mutex, mpsc};
use tracing::{debug, info, warn};

use crate::protocol::game;
use crate::threads::{processor::connection, server::server};

pub mod protocol;
pub mod threads;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to bind the TCP Connection
    #[arg(short, long, default_value_t = 5051)]
    port: u16,
}

fn main() -> ! {
    dotenvy::dotenv().expect("[MAIN] Failed to load .env file");
    tracing_config::init!();

    let args = Args::parse();

    let address = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&address).expect("[MAIN] Failed to bind to address");

    info!("[MAIN] Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(rx));

    // Build the game map
    let path = env::var("MAP_FILEPATH").expect("[MAIN] MAP_FILEPATH must be set.");
    let file = File::open(path).expect("[MAIN] Failed to open map file!");
    let mut rooms = game::build(file).expect("[MAIN] Failed to build map from file");

    // Start the server thread with the map
    info!("[MAIN] Parsed map successfully");

    let _ = std::thread::spawn(move || {
        server(receiver, &mut rooms);
    });

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("[MAIN] New connection: {}", addr);

                let stream = Arc::new(stream);
                let sender = tx.clone();

                // Handle the connection in a separate thread
                let client_h = std::thread::spawn(move || {
                    connection(stream, sender);
                });

                debug!("[MAIN] Spawned client thread: {:?}", client_h.thread().id());
            }
            Err(e) => {
                warn!("[MAIN] Error accepting connection: {}", e);
            }
        }
    }
}
