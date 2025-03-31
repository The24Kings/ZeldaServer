use std::sync::{Arc, Mutex, mpsc};
use std::net::TcpListener;

use crate::threads::{processor::connection, server::server};

pub mod protocol;
pub mod threads;

fn main() {
    let address = "0.0.0.0:5051";
    let listener = TcpListener::bind(address).expect("Failed to bind to address");

    println!("Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(rx));

    /*
        Build the game map from a JSON file
        the map should compile into a struct:

        Map {
            tiles: Vec<Tile>,
            players: Vec<Player>,
            monsters: Vec<Monster>,
            desc: String
        }
        Tile {
            id: u8,                         // Unique ID
            title: String,                  // Title of the tile (also needs to be unique)
            exits: Vec<Exit>,               // Possible exits (Points to other tiles)
            players: Vec<Player_index>,     // All players currently on the tile (index to the player list in the map)
            monsters: Vec<Monster_index>,   // All monsters currently on the tile (index to the monster list in the map)
            desc: String
        }    
        Exit {
            id: u8,         // Same as the Tile id we are going to
            title: String,  // Title of the tile we are going to
            desc: String    // May be an abbreviated description of the Tile we are going to
        }

        This struct is mutable and should be passed to the server thread
        The server thread should be able to modify the map according to packets sent by the client
     */

    // Start the server thread
    std::thread::spawn(move || {
        server(receiver);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {:?}", stream.peer_addr());
                let stream = Arc::new(stream);
                let sender = tx.clone();

                // Handle the connection in a separate thread
                std::thread::spawn(move || {
                    connection(stream, sender);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
