use lurk_lcsc::PktStart;
use lurk_lcsc::{CharacterFlags, LurkError};
use lurk_lcsc::{PktError, PktRoom};
use lurk_lcsc::{send_character, send_error, send_room};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::map;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_start(&mut self, author: Arc<TcpStream>, content: PktStart) {
        info!("Received: {}", content);

        // Find the player in the map
        let player = match map::player_from_stream(&mut self.players, author.clone()) {
            Some((name, player)) => {
                info!("Found player '{}'", name);
                player
            }
            None => {
                error!("Unable to find player in map");
                return;
            }
        };

        if !player.flags.is_ready() {
            send_error!(
                author.clone(),
                PktError::new(LurkError::NOTREADY, "Supply of valid player first!")
            );

            return;
        }

        // ================================================================================
        // Activate the character and send the information off to client
        // ================================================================================
        player.flags |= CharacterFlags::STARTED;

        let player = player.clone(); // End mutable borrow of player

        send_character!(author.clone(), player);

        // ================================================================================
        // Send the starting room and connections to the client
        // ================================================================================
        let room = match self.rooms.get(&0) {
            Some(room) => room,
            None => {
                error!("Unable to find room in map");
                return;
            }
        };

        self.alert_room(room, &player);
        self.broadcast(format!("{} has started the game!", player.name));

        let room = match self.rooms.get_mut(&0) {
            Some(room) => room,
            None => {
                error!("Unable to find room in map");
                return;
            }
        };

        info!("Adding player to starting room");

        room.players.push(player.name);

        send_room!(author.clone(), PktRoom::from(room.clone()));

        self.send_connections(&author, 0);

        // ================================================================================
        // Send the all players and monsters in the room excluding the author
        // ================================================================================
        if let Some(room) = self.rooms.get(&0) {
            self.send_room_contents(&author, room);
        }
    }
}
