use std::sync::{Arc, mpsc::Sender};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::protocol::packet::{pkt_game, pkt_leave, pkt_version};
use crate::protocol::{Protocol, Stream, client::Client, pkt_type::PktType};

pub fn connection(stream: Stream, sender: Sender<Protocol>, config: Arc<Config>) {
    let client = Client::new(stream.clone(), sender);

    let description = std::fs::read_to_string(&config.description_path)
        .expect("[CONNECT] Failed to read description file!");

    // Send the initial game info to the client
    Protocol::Version(
        stream.clone(),
        pkt_version::Version {
            message_type: PktType::VERSION,
            major_rev: config.major_rev,
            minor_rev: config.minor_rev,
            extension_len: 0,
            extensions: None,
        },
    )
    .send()
    .unwrap_or_else(|e| {
        error!("[CONNECT] Failed to send version packet: {}", e);
        return; // This is a critical error, so we return
    });

    Protocol::Game(
        stream.clone(),
        pkt_game::Game {
            message_type: PktType::GAME,
            initial_points: config.initial_points,
            stat_limit: config.stat_limit,
            description_len: description.len() as u16,
            description,
        },
    )
    .send()
    .unwrap_or_else(|e| {
        error!("[CONNECT] Failed to send game packet: {}", e);
        return; // This is a critical error, so we return
    });

    // Main loop to read packets from the client
    loop {
        match client.read() {
            Ok(_) => {
                info!("[READ] Packet read successfully.");
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        error!("[READ] Connection reset: {}. Terminating thread.", e);
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        error!("[READ] Connection aborted: {}. Terminating thread.", e);
                    }
                    std::io::ErrorKind::NotConnected => {
                        error!("[READ] Not connected: {}. Terminating thread.", e);
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        error!("[READ] Broken pipe: {}. Terminating thread.", e);
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        error!("[READ] Unexpected EOF: {}. Terminating thread.", e);
                    }
                    std::io::ErrorKind::Unsupported => {
                        error!("[READ] Unsupported operation: {}. Terminating thread.", e);
                    }
                    _ => {
                        warn!("[READ] Non-terminal error: '{}'. Continuing.", e);
                        continue; // Continue processing other packets
                    }
                }

                // If we reach here, it means the connection was closed
                // Ensure the server thread is notified of the disconnection
                client
                    .sender
                    .send(Protocol::Leave(stream.clone(), pkt_leave::Leave::default()))
                    .unwrap_or_else(|_| {
                        error!("[CONNECT] Failed to send leave packet");
                    });

                break;
            }
        }
    }
}
