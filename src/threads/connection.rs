use lurk_lcsc::{PktGame, PktLeave, PktType, PktVersion, Protocol, send_game, send_version};
use std::io::ErrorKind::{UnexpectedEof, Unsupported};
use std::net::TcpStream;
use std::sync::{Arc, mpsc::Sender};
use tracing::{error, info, warn};

use crate::logic::{ExtendedProtocol, config::Config};
use crate::send_ext_base;

pub fn connection(stream: Arc<TcpStream>, sender: Sender<ExtendedProtocol>, config: Arc<Config>) {
    let description = std::fs::read_to_string(&config.description_path)
        .expect("Failed to read description file!");

    // Send the initial game info to the client
    send_version!(
        stream.clone(),
        PktVersion {
            packet_type: PktType::VERSION,
            major_rev: config.major_rev,
            minor_rev: config.minor_rev,
            extensions_len: 0,
            extensions: None,
        }
    );

    send_game!(
        stream.clone(),
        PktGame {
            packet_type: PktType::GAME,
            initial_points: config.initial_points,
            stat_limit: config.stat_limit,
            description_len: description.len() as u16,
            description: Box::from(description),
        }
    );

    // Main loop to read packets from the client
    loop {
        match Protocol::recv(&stream) {
            Ok(pkt) => {
                info!("Packet read successfully");

                // Try to send the packet
                match sender.send(ExtendedProtocol::Base(pkt)) {
                    Ok(()) => continue, // Don't fallout to graceful exit
                    Err(e) => {
                        error!("Failed to send packet: {}", e);
                    }
                };
            }
            Err(e) => {
                match e.kind() {
                    UnexpectedEof | Unsupported => {
                        error!("'{:?}' -> {}. Terminating.", e.kind(), e);
                    }
                    _ => {
                        warn!("'{:?}' -> {}. Continuing.", e.kind(), e);
                        continue; // Non-terminal; Continue processing other packets
                    }
                }
            }
        };

        // Exit gracefully
        send_ext_base!(sender, Protocol::Leave(stream.clone(), PktLeave::default()));
        break;
    }

    info!("Connection handler exiting.");
}
