use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::protocol::client::Client;
use crate::protocol::packet::{game::Game, leave::Leave, version::Version};
use crate::protocol::{Type, send};

pub fn connection(stream: Arc<TcpStream>, initial_points: u16, stat_limit: u16, sender: Sender<Type>) {
    let client = Client::new(stream.clone(), sender, None);

    let description = std::fs::read_to_string("src/content/desc.txt")
        .expect("[CONNECTION] Failed to read description file!");

    // Send the initial game info to the client
    send(Type::Version(
        stream.clone(),
        Version {
            message_type: 14,
            major_rev: 2,
            minor_rev: 3,
            extension_len: 0,
            extensions: None,
        }
    ))
    .unwrap_or_else(|e| {
        eprintln!("[CONNECTION] Failed to send version packet: {}", e);
        return; // This is a critical error, so we return
    });

    send(Type::Game(
        stream.clone(),
        Game {
            message_type: 11,
            initial_points,
            stat_limit,
            description_len: description.len() as u16,
            description,
        }
    ))
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
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        eprintln!("[CONNECTION] Connection aborted. Terminating thread.");
                    }
                    std::io::ErrorKind::NotConnected => {
                        eprintln!("[CONNECTION] Not connected. Terminating thread.");
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        eprintln!("[CONNECTION] Broken pipe. Terminating thread.");
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        eprintln!("[CONNECTION] Unexpected EOF. Terminating thread.");
                    }
                    std::io::ErrorKind::Unsupported => {
                        eprintln!("[CONNECTION] Unsupported operation. Terminating thread.");
                    }
                    _ => {
                        eprintln!("[CONNECTION] Non-terminal error: '{}'. Continuing.", e);
                        continue; // Continue processing other packets
                    }
                }

                // If we reach here, it means the connection was closed
                // Ensure the server thread is notified of the disconnection
                client
                    .sender
                    .send(Type::Leave(stream.clone(), Leave::default()))
                    .unwrap_or_else(|_| {
                        eprintln!("[CONNECTION] Failed to send leave packet");
                    });

                break;
            }
        }
    }
}
