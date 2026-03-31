use lurk_lcsc::LurkError;
use lurk_lcsc::PktChangeRoom;
use lurk_lcsc::{PktError, PktRoom};
use lurk_lcsc::{send_character, send_error, send_room};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::map;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_change_room(&mut self, author: Arc<TcpStream>, content: PktChangeRoom) {
        info!("Received: {}", content);

        // Find the player in the map
        let player = match map::player_from_stream(&mut self.players, author.clone()) {
            Some((_, player)) => player,
            None => {
                error!("Unable to find player in map");
                return;
            }
        };

        if !GameState::ensure_started(player, &author) {
            return;
        }

        let cur_room_id = player.current_room;
        let nxt_room_id = content.room_number;

        // ================================================================================
        // Check to make sure the player exists, is in the given room, and can move to the
        // given connection. Shuffle the player around to the next room and send data.
        // ================================================================================
        if cur_room_id == nxt_room_id {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADROOM, "Player is already in the room")
            );

            return;
        }
        // Check if the room is a valid connection
        let cur_room = match self.rooms.get_mut(&cur_room_id) {
            Some(room) => room,
            None => {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::BADROOM, "Room not found!")
                );

                return;
            }
        };

        match cur_room.connections.get(&nxt_room_id) {
            Some(exit) => {
                info!("Found connection: '{}'", exit.title);
            }
            None => {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::BADROOM, "Invalid connection!")
                );

                return;
            }
        }

        info!("Setting current room to: {}", nxt_room_id);
        player.current_room = nxt_room_id;

        info!("Removing player from old room");
        cur_room.players.retain(|name| *name != player.name);

        let cur_room = cur_room.clone(); // End mutable borrow of cur_room

        // Find the next room in the map, add the player, and send it off
        let new_room = match self.rooms.get_mut(&nxt_room_id) {
            Some(room) => room,
            None => {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::BADROOM, "Room not found!")
                );

                return;
            }
        };

        info!("Adding player to new room");
        new_room.players.push(player.name.clone());

        send_room!(author.clone(), PktRoom::from(new_room.clone()));

        let new_room = new_room.clone(); // End mutable borrow of new_room

        // ================================================================================
        // Update the player data and send it to the client
        // ================================================================================
        info!("Updating player room");

        // Send the updated character back to the client
        send_character!(author.clone(), player.clone());

        // ================================================================================
        // Update info for all other connected clients
        // ================================================================================
        let player = player.clone(); // End mutable borrow of player

        self.alert_room(&cur_room, &player);
        self.alert_room(&new_room, &player);

        // ================================================================================
        // Send all connections and room contents to the client
        // ================================================================================
        self.send_connections(&author, nxt_room_id);
        self.send_room_contents(&author, &new_room);
    }
}
