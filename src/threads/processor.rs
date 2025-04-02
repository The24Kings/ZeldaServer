use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::protocol::client::Client;
use crate::protocol::packet::{game::Game, version::Version};
use crate::protocol::{Type, send};

pub fn connection(stream: Arc<TcpStream>, sender: Sender<Type>) {
    let client = Client::new(stream.clone(), sender);

    // Send the initial game info to the client
    send(Type::Version(Version {
        author: Some(stream.clone()),
        message_type: 14,
        major_rev: 2,
        minor_rev: 3,
        extension_len: 0,
        extensions: None,
    }))
    .unwrap_or_else(|e| {
        eprintln!("[CONNECTION] Failed to send version packet: {}", e);
        return; // This is a critical error, so we return
    });

    send(Type::Game(Game {
        author: Some(stream.clone()),
        message_type: 11,
        initial_points: 100,
        stat_limit: 65225,
        description_len: 11,
        description: "Hello world".to_string(),
    }))
    .unwrap_or_else(|e| {
        eprintln!("[CONNECTION] Failed to send game packet: {}", e);
        return; // This is a critical error, so we return
    });

    // Main loop to read packets from the client
    loop {
        match client.read() {
            Ok(_) => {
                println!("[CONNECTION] Packet read successfully");
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        eprintln!("[CONNECTION] Connection reset by peer. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        eprintln!("[CONNECTION] Connection aborted. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::NotConnected => {
                        eprintln!("[CONNECTION] Not connected. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        eprintln!("[CONNECTION] Broken pipe. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        eprintln!("[CONNECTION] Unexpected EOF. Terminating thread.");
                        break;
                    }
                    _ => {
                        eprintln!("[CONNECTION] Non-terminal error: '{}'. Continuing.", e);
                        // Continue processing other packets
                        continue;
                    }
                }
            }
        }
    }
}
