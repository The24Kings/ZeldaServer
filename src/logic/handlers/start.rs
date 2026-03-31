use lurk_lcsc::PktStart;
use lurk_lcsc::{CharacterFlags, LurkError};
use lurk_lcsc::{PktError, PktRoom};
use lurk_lcsc::{send_error, send_room, send_to};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::map;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_start(&mut self, author: Arc<TcpStream>, content: PktStart) {
        info!("Received: {}", content);

        // ================================================================================
        // Phase 1: Find player, validate, activate, extract name
        // ================================================================================
        let player_name = {
            let Some((name, player)) = map::player_from_stream(&mut self.players, author.clone())
            else {
                error!("Unable to find player in map");
                return;
            };
            info!("Found player '{}'", name);

            if !player.flags.is_ready() {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::NOTREADY, "Supply of valid player first!")
                );

                return;
            }

            player.flags |= CharacterFlags::STARTED;

            player.name.clone()
        };

        // Send updated character
        if let Some(player) = self.players.get(&player_name) {
            let _ = send_to(author.as_ref(), player);
        }

        // ================================================================================
        // Alert room and broadcast (shared borrows only)
        // ================================================================================
        if let Some(room) = self.rooms.get(&0)
            && let Some(player) = self.players.get(&player_name)
        {
            self.alert_room(room, player);
        }
        self.broadcast(format!("{} has started the game!", player_name));

        // ================================================================================
        // Mutate: add player to starting room
        // ================================================================================
        if let Some(room) = self.rooms.get_mut(&0) {
            info!("Adding player to starting room");
            room.players.insert(player_name);
        }

        // ================================================================================
        // Send room, connections, and contents (shared borrows)
        // ================================================================================
        if let Some(room) = self.rooms.get(&0) {
            send_room!(author.clone(), PktRoom::from(room));
        }

        self.send_connections(&author, 0);

        if let Some(room) = self.rooms.get(&0) {
            self.send_room_contents(&author, room);
        }
    }
}
