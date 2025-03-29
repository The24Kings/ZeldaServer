use std::sync::Arc;

use crate::threads::client::client;

pub mod protocol;
pub mod threads;

fn main() {
    let address = "0.0.0.0:5051";
    let listener = std::net::TcpListener::bind(address).expect("Failed to bind to address");

    println!("Listening on {address}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let stream = Arc::new(stream);
                
                // Handle the connection in a separate thread
                std::thread::spawn(move || {
                    client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
