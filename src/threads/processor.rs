use std::env;
use std::sync::mpsc::Sender;

use crate::protocol::packet::{pkt_game, pkt_leave, pkt_version};
use crate::protocol::{ServerMessage, Stream, client::Client, send};

pub fn connection(
    stream: Stream,
    initial_points: u16,
    stat_limit: u16,
    sender: Sender<ServerMessage>,
) {
    let client = Client::new(stream.clone(), sender);

    let filepath = env::var("DESC_FILEPATH").expect("[CONNECTION] DESC_FILEPATH must be set.");
    let description =
        std::fs::read_to_string(filepath).expect("[CONNECTION] Failed to read description file!");

    // Send the initial game info to the client
    send(ServerMessage::Version(
        stream.clone(),
        pkt_version::Version {
            message_type: 14,
            major_rev: env::var("MAJOR_REV")
                .expect("[CONNECTION] MAJOR_REV must be set.")
                .parse()
                .expect("[CONNECTION] Failed to parse MAJOR_REV"),
            minor_rev: env::var("MINOR_REV")
                .expect("[CONNECTION] MINOR_REV must be set.")
                .parse()
                .expect("[CONNECTION] Failed to parse MINOR_REV"),
            extension_len: 0,
            extensions: None,
        },
    ))
    .unwrap_or_else(|e| {
        eprintln!("[CONNECTION] Failed to send version packet: {}", e);
        return; // This is a critical error, so we return
    });

    send(ServerMessage::Game(
        stream.clone(),
        pkt_game::Game {
            message_type: 11,
            initial_points,
            stat_limit,
            description_len: description.len() as u16,
            description,
        },
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
                    .send(ServerMessage::Leave(
                        stream.clone(),
                        pkt_leave::Leave::default(),
                    ))
                    .unwrap_or_else(|_| {
                        eprintln!("[CONNECTION] Failed to send leave packet");
                    });

                break;
            }
        }
    }
}
