use lurk_lcsc::LurkError;
use lurk_lcsc::PktCharacter;
use lurk_lcsc::PktMessage;
use lurk_lcsc::{PktConnection, PktError, send_error, send_to};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Arc;
use tracing::error;
use tracing::info;
use tracing::trace;

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
                let _ = send_to(author.as_ref(), player);
            }
        }

        // Send all monsters in the room
        if let Some(monsters) = &room.monsters {
            for monster in monsters {
                let pkt = PktCharacter::from(monster);
                let _ = send_to(author.as_ref(), &pkt);
            }
        }
    }

    /// Send all connection exits for a room to a client.
    pub fn send_connections(&self, author: &Arc<TcpStream>, room_id: u16) {
        let Some(room) = self.rooms.get(&room_id) else {
            error!("No exits for room {}", room_id);
            return;
        };

        for conn in room.connections.values() {
            let pkt = PktConnection::from(conn);
            let _ = send_to(author.as_ref(), &pkt);
        }
    }

    /// Find a player by their TCP stream.
    pub fn player_by_stream(
        &mut self,
        stream: &Arc<TcpStream>,
    ) -> Option<(&Arc<str>, &mut PktCharacter)> {
        map::player_from_stream(&mut self.players, stream.clone())
    }

    /// Internal helper: send a constructed message to each named player.
    fn send_to_players<'a>(
        players: &'a HashMap<Arc<str>, PktCharacter>,
        names: impl Iterator<Item = &'a Arc<str>>,
        msg_fn: impl Fn(&Arc<str>) -> PktMessage,
    ) {
        for name in names {
            let Some(player) = players.get(name) else {
                continue;
            };
            let Some(author) = player.author.as_ref() else {
                continue;
            };

            let msg = msg_fn(name);
            let _ = send_to(author.as_ref(), &msg);
        }
    }

    /// Broadcast a message to all connected players.
    pub fn broadcast(&self, message: String) {
        info!("Sending message: {}", message);
        GameState::send_to_players(&self.players, self.players.keys(), |name| {
            PktMessage::server(name, &message)
        });
    }

    /// Send a message to all players in a specific room.
    pub fn message_room(&self, room: &Room, message: String, narration: bool) {
        info!(
            "[ROOM MESSAGE] Messaging room {}: {}",
            room.room_number, message
        );
        GameState::send_to_players(&self.players, room.players.iter(), |name| {
            if narration {
                PktMessage::narrator(name, &message)
            } else {
                PktMessage::server(name, &message)
            }
        });
    }

    /// Alert all players in the current room of a character change by sending a Character packet
    /// to each player in the room.
    pub fn alert_room(&self, room: &Room, alert: &PktCharacter) {
        info!("Alerting players about: '{}'", alert.name);

        room.players.iter().for_each(|name| {
            trace!("Alerting player: '{}'", name);

            let Some(player) = self.players.get(name) else {
                return;
            };
            let Some(author) = player.author.as_ref() else {
                return;
            };

            let _ = send_to(author.as_ref(), alert);
        });
    }
}
