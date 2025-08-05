use std::env;
use std::sync::mpsc::Sender;
use tracing::{error, info, warn};

use crate::protocol::packet::{pkt_game, pkt_leave, pkt_version};
use crate::protocol::{ServerMessage, Stream, client::Client, pkt_type::PktType};

pub fn connection(
    stream: Stream,
    initial_points: u16,
    stat_limit: u16,
    sender: Sender<ServerMessage>,
) {
    let client = Client::new(stream.clone(), sender);

    let filepath = env::var("DESC_FILEPATH").expect("DESC_FILEPATH must be set.");
    let description = std::fs::read_to_string(filepath).expect("Failed to read description file!");

    // Send the initial game info to the client
    ServerMessage::Version(
        stream.clone(),
        pkt_version::Version {
            message_type: PktType::Version,
            major_rev: env::var("MAJOR_REV")
                .expect("MAJOR_REV must be set.")
                .parse()
                .expect("Failed to parse MAJOR_REV"),
            minor_rev: env::var("MINOR_REV")
                .expect("MINOR_REV must be set.")
                .parse()
                .expect("Failed to parse MINOR_REV"),
            extension_len: 0,
            extensions: None,
        },
    )
    .send()
    .unwrap_or_else(|e| {
        error!("Failed to send version packet: {}", e);
        return; // This is a critical error, so we return
    });

    ServerMessage::Game(
        stream.clone(),
        pkt_game::Game {
            message_type: PktType::Game,
            initial_points,
            stat_limit,
            description_len: description.len() as u16,
            description,
        },
    )
    .send()
    .unwrap_or_else(|e| {
        error!("Failed to send game packet: {}", e);
        return; // This is a critical error, so we return
    });

    // Main loop to read packets from the client
    loop {
        match client.read() {
            Ok(_) => {
                info!("Packet read successfully");
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        error!("Connection reset by peer. Terminating thread.");
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        error!("Connection aborted. Terminating thread.");
                    }
                    std::io::ErrorKind::NotConnected => {
                        error!("Not connected. Terminating thread.");
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        error!("Broken pipe. Terminating thread.");
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        error!("Unexpected EOF. Terminating thread.");
                    }
                    std::io::ErrorKind::Unsupported => {
                        error!("Unsupported operation. Terminating thread.");
                    }
                    _ => {
                        warn!("Non-terminal error: '{}'. Continuing.", e);
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
                        error!("Failed to send leave packet");
                    });

                break;
            }
        }
    }
}
