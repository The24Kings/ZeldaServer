use lurk_lcsc::{LurkError, PktChangeRoom, PktError, PktRoom};
use lurk_lcsc::{send_error, send_room, send_to};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::GameState;

impl GameState {
    pub fn handle_change_room(&mut self, author: Arc<TcpStream>, content: PktChangeRoom) {
        info!("Received: {}", content);

        // ================================================================================
        // Phase 1: Find player, validate, extract IDs
        // ================================================================================
        let (player_name, cur_room_id) = {
            let Some((_, player)) = self.player_from_stream(&author) else {
                error!("Unable to find player in map");
                return;
            };

            if !GameState::ensure_started(player, &author) {
                return;
            }

            (player.name.clone(), player.current_room)
        }; // Mutable borrow of self.players dropped

        let nxt_room_id = content.room_number;

        if cur_room_id == nxt_room_id {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADROOM, "Player is already in the room")
            );

            return;
        }

        // Validate connection exists
        let Some(room) = self.rooms.get(&cur_room_id) else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADROOM, "Room not found!")
            );

            return;
        };

        let Some(exit) = room.connections.get(&nxt_room_id) else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADROOM, "Invalid connection!")
            );

            return;
        };

        info!("Found connection: '{}'", exit.title);

        // ================================================================================
        // Phase 2: Apply the changes to the player and room
        // ================================================================================
        if let Some(player) = self.players.get_mut(&player_name) {
            info!("Setting current room to: {}", nxt_room_id);
            player.current_room = nxt_room_id;
        }

        if let Some(cur_room) = self.rooms.get_mut(&cur_room_id) {
            info!("Removing player from old room");
            cur_room.players.retain(|name| *name != player_name);
        }

        if let Some(new_room) = self.rooms.get_mut(&nxt_room_id) {
            info!("Adding player to new room");
            new_room.players.insert(player_name.clone());
        }

        // ================================================================================
        // Phase 3: Alert and send the updated data to the client
        // ================================================================================
        if let Some(new_room) = self.rooms.get(&nxt_room_id) {
            send_room!(author.clone(), PktRoom::from(new_room));
        }

        if let Some(player) = self.players.get(&player_name) {
            let _ = send_to(author.as_ref(), player);
        }

        // Alert old and new rooms about the player change
        if let Some(player) = self.players.get(&player_name) {
            if let Some(cur_room) = self.rooms.get(&cur_room_id) {
                self.alert_room(cur_room, player);
            }
            if let Some(new_room) = self.rooms.get(&nxt_room_id) {
                self.alert_room(new_room, player);
            }
        }

        self.send_connections(&author, nxt_room_id);

        if let Some(new_room) = self.rooms.get(&nxt_room_id) {
            self.send_room_contents(&author, new_room);
        }
    }
}
