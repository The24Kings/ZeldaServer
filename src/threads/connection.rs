use lurk_lcsc::{PktGame, PktLeave, PktType, PktVersion, Protocol};
use std::io::ErrorKind::{UnexpectedEof, Unsupported};
use std::net::TcpStream;
use std::sync::{Arc, mpsc::Sender};
use tracing::{error, info, warn};

use crate::logic::{ExtendedProtocol, config::Config};

pub fn connection(stream: Arc<TcpStream>, sender: Sender<ExtendedProtocol>, config: Arc<Config>) {
    let description = std::fs::read_to_string(&config.description_path)
        .expect("[CONNECT] Failed to read description file!");

    // Send the initial game info to the client
    Protocol::Version(
        stream.clone(),
        PktVersion {
            message_type: PktType::VERSION,
            major_rev: config.major_rev,
            minor_rev: config.minor_rev,
            extension_len: 0,
            extensions: None,
        },
    )
    .send()
    .expect("[CONNECT] Failed to send version packet");

    Protocol::Game(
        stream.clone(),
        PktGame {
            message_type: PktType::GAME,
            initial_points: config.initial_points,
            stat_limit: config.stat_limit,
            description_len: description.len() as u16,
            description: Box::from(description),
        },
    )
    .send()
    .expect("[CONNECT] Failed to send game packet");

    // Main loop to read packets from the client
    loop {
        match Protocol::recv(&stream) {
            Ok(pkt) => {
                info!("[READ] Packet read successfully");

                // Try to send the packet
                match sender.send(ExtendedProtocol::Base(pkt)) {
                    Ok(()) => continue, // Don't fallout to graceful exit
                    Err(e) => {
                        error!("[READ] Failed to send packet: {}", e);
                    }
                };
            }
            Err(e) => {
                match e.kind() {
                    UnexpectedEof | Unsupported => {
                        error!("[READ] '{:?}' -> {}. Terminating.", e.kind(), e);
                    }
                    _ => {
                        warn!("[READ] '{:?}' -> {}. Continuing.", e.kind(), e);
                        continue; // Non-terminal; Continue processing other packets
                    }
                }
            }
        };

        // Exit gracefully
        sender
            .send(ExtendedProtocol::Base(Protocol::Leave(
                stream.clone(),
                PktLeave::default(),
            )))
            .unwrap_or_else(|_| {
                error!("[CONNECT] Failed to send leave packet");
            });

        break;
    }
}
