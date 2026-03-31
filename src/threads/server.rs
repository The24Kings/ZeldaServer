use lurk_lcsc::Protocol;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::time::Instant;
use tracing::{debug, warn};

use crate::logic::state::GameState;
use crate::logic::{ExtendedProtocol, config::Config, map};

pub fn server(
    receiver: Arc<Mutex<Receiver<ExtendedProtocol>>>,
    config: Arc<Config>,
    rooms: HashMap<u16, map::Room>,
) -> ! {
    let mut state = GameState::new(rooms, config);

    loop {
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                warn!("Error receiving packet: {}", e);
                continue;
            }
        };

        let start = Instant::now();

        match packet {
            ExtendedProtocol::Base(Protocol::Message(author, content)) => {
                state.handle_message(author, content);
            }
            ExtendedProtocol::Base(Protocol::ChangeRoom(author, content)) => {
                state.handle_change_room(author, content);
            }
            ExtendedProtocol::Base(Protocol::Fight(author, content)) => {
                state.handle_fight(author, content);
            }
            ExtendedProtocol::Base(Protocol::PVPFight(author, content)) => {
                state.handle_pvp_fight(author, content);
            }
            ExtendedProtocol::Base(Protocol::Loot(author, content)) => {
                state.handle_loot(author, content);
            }
            ExtendedProtocol::Base(Protocol::Start(author, content)) => {
                state.handle_start(author, content);
            }
            ExtendedProtocol::Base(Protocol::Character(author, content)) => {
                state.handle_character(author, content);
            }
            ExtendedProtocol::Base(Protocol::Leave(author, content)) => {
                state.handle_leave(author, content);
            }
            ExtendedProtocol::Base(_) => {} // Ignore all other packets
            ExtendedProtocol::Command(action) => {
                state.handle_command(action);
            }
        }

        let end = Instant::now();
        let delta = end.duration_since(start);
        let secs = delta.as_secs();
        let nanos = delta.subsec_nanos();

        debug!("Took: {secs}.{nanos} seconds to process packet.");
    }
}
