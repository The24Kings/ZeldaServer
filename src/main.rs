use std::fs::File;
use std::sync::{Arc, Mutex, mpsc};
use std::net::TcpListener;

use protocol::map::Map;

use crate::threads::{processor::connection, server::server};

pub mod protocol;
pub mod threads;

fn main() {
    let address = "0.0.0.0:5051";
    let listener = TcpListener::bind(address).expect("[MAIN] Failed to bind to address");

    println!("Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(rx));

    // Build the game map
    let file = File::open("src/content/game.json").expect("[MAIN] Failed to open map file!");
    let map = Map::build(file);

    let initial_points = map.as_ref().map(|m| m.init_points).unwrap_or(100);
    let stat_limit = map.as_ref().map(|m| m.stat_limit).unwrap_or(65525);

    // Start the server thread with the map
    match map {
        Ok(mut map) => {
            println!("[MAIN] Parsed map successfully");
            
            std::thread::spawn(move || {
                server(receiver, &mut map);
            });
        }
        Err(e) => {
            eprintln!("[MAIN] Error parsing map: {}", e);
        }
    }

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
