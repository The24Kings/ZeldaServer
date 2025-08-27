use std::io::ErrorKind::{BrokenPipe, UnexpectedEof, Unsupported};
use std::sync::{Arc, mpsc::Sender};
use tracing::{error, info, warn};

use crate::logic::config::Config;
use crate::protocol::packet::{game, leave, version};
use crate::protocol::{Protocol, Stream, pkt_type::PktType};
use crate::threads::client::Client;

pub fn connection(stream: Stream, sender: Sender<Protocol>, config: Arc<Config>) {
    let client = Client::new(stream.clone(), sender);

    let description = std::fs::read_to_string(&config.description_path)
        .expect("[CONNECT] Failed to read description file!");

    // Send the initial game info to the client
    Protocol::Version(
        client.stream.clone(),
        version::PktVersion {
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
        client.stream.clone(),
        game::PktGame {
            message_type: PktType::GAME,
            initial_points: config.initial_points,
            stat_limit: config.stat_limit,
            description_len: description.len() as u16,
            description: Box::from(description),
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
                    BrokenPipe | UnexpectedEof | Unsupported => {
                        error!("[READ] '{:?}' -> {}. Terminating.", e.kind(), e);
                    }
                    _ => {
                        warn!("[READ] '{:?}' -> {}. Continuing.", e.kind(), e);
                        continue; // Continue processing other packets
                    }
                }

                // Exit gracefully
                client
                    .sender
                    .send(Protocol::Leave(stream.clone(), leave::PktLeave::default()))
                    .unwrap_or_else(|_| {
                        error!("[CONNECT] Failed to send leave packet");
                    });

                break;
            }
        }
    }
}
