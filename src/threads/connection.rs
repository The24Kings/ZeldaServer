use lurk_lcsc::{PktGame, PktLeave, PktType, PktVersion, Protocol};
use lurk_lcsc::{send_game, send_version};
use std::io::ErrorKind::{UnexpectedEof, Unsupported};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::logic::{Config, GameSender};

pub fn connection(stream: Arc<TcpStream>, sender: GameSender, config: Arc<Config>) {
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
            description_len: config.description.len() as u16,
            description: config.description.clone(),
        }
    );

    // Main loop to read packets from the client
    loop {
        match Protocol::recv(&stream) {
            Ok(pkt) => {
                info!("Packet read successfully");

                sender.send_base(pkt);
                continue; // Don't fallout to graceful exit
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
        sender.send_base(Protocol::Leave(stream.clone(), PktLeave::default()));
        break;
    }

    info!("Connection handler exiting.");
}
