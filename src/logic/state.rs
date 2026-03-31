use lurk_lcsc::LurkError;
use lurk_lcsc::{
    PktCharacter, PktConnection, PktError, send_character, send_connection, send_error,
};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Arc;
use tracing::error;

use crate::logic::config::Config;
use crate::logic::map::{self, Room};

/// Central game state holding all players, rooms, and server configuration.
pub struct GameState {
    pub players: HashMap<Arc<str>, PktCharacter>,
    pub rooms: HashMap<u16, Room>,
    pub config: Arc<Config>,
}

impl GameState {
    pub fn new(rooms: HashMap<u16, Room>, config: Arc<Config>) -> Self {
        Self {
            players: HashMap::new(),
            rooms,
            config,
        }
    }

    /// Check that a player is started and ready. Sends an error to the author if not.
    /// Returns `true` if the player is started and ready.
    pub fn ensure_started(player: &PktCharacter, author: &Arc<TcpStream>) -> bool {
        if !player.flags.is_started() && !player.flags.is_ready() {
            send_error!(
                author.clone(),
                PktError::new(LurkError::NOTREADY, "Start the game first!")
            );
            return false;
        }
        true
    }

    /// Send players, and monsters to a client.
    pub fn send_room_contents(&self, author: &Arc<TcpStream>, room: &Room) {
        // Send all players in the room
        for name in &room.players {
            if let Some(player) = self.players.get(name) {
                send_character!(author.clone(), player.clone());
            }
        }

        // Send all monsters in the room
        if let Some(monsters) = &room.monsters {
            for monster in monsters {
                send_character!(author.clone(), PktCharacter::from(monster));
            }
        }
    }

    /// Send all connection exits for a room to a client.
    pub fn send_connections(&self, author: &Arc<TcpStream>, room_id: u16) {
        let connections = match self.rooms.get(&room_id) {
            Some(room) => &room.connections,
            None => {
                error!("No exits for room {}", room_id);
                return;
            }
        };

        for conn in connections.values() {
            send_connection!(author.clone(), PktConnection::from(conn.clone()));
        }
    }

    /// Find a player by their TCP stream.
    pub fn player_by_stream(
        &mut self,
        stream: &Arc<TcpStream>,
    ) -> Option<(&Arc<str>, &mut PktCharacter)> {
        map::player_from_stream(&mut self.players, stream.clone())
    }

    /// Broadcast a message to all connected players.
    pub fn broadcast(&self, message: String) {
        map::broadcast(&self.players, message);
    }

    /// Send a message to all players in a specific room.
    pub fn message_room(&self, room: &Room, message: String, narration: bool) {
        map::message_room(&self.players, room, message, narration);
    }

    /// Alert all players in a room about a character change.
    pub fn alert_room(&self, room: &Room, alert: &PktCharacter) {
        map::alert_room(&self.players, room, alert);
    }
}
