use clap::Parser;
use dotenv::dotenv;
use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::sync::{Arc, Mutex, mpsc};

use protocol::map::Map;

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

fn main() {
    dotenv().ok();

    let args = Args::parse();

    let address = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&address).expect("[MAIN] Failed to bind to address");

    println!("Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(rx));

    // Build the game map
    let path = env::var("MAP_FILEPATH").expect("[MAIN] MAP_FILEPATH must be set.");
    let file = File::open(path).expect("[MAIN] Failed to open map file!");
    let mut map = Map::build(file).expect("[MAIN] Failed to build map from file");

    let initial_points = map.init_points;
    let stat_limit = map.stat_limit;

    // Start the server thread with the map
    println!("[MAIN] Parsed map successfully");

    std::thread::spawn(move || {
        server(receiver, &mut map);
    });

    // Listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("[MAIN] New connection: {:?}", stream.peer_addr());
                let stream = Arc::new(stream);
                let sender = tx.clone();

                // Handle the connection in a separate thread
                std::thread::spawn(move || {
                    connection(stream, initial_points, stat_limit, sender);
                });
            }
            Err(e) => {
                eprintln!("[MAIN] Error accepting connection: {}", e);
            }
        }
    }
}
