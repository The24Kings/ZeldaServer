use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, sync::Arc};
use tracing::{debug, error, info, warn};

use crate::protocol::{
    Protocol, Stream,
    packet::{pkt_character, pkt_message},
    pkt_type::PktType,
};

#[derive(Debug)]
pub struct Map {
    pub rooms: Vec<Room>, //TODO: Also change to HasMap and use the room id as the key
    pub players: HashMap<String, pkt_character::Character>,
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: String,
    pub connections: Vec<Connection>,
    pub desc: String,
    pub players: Vec<String>,
    pub monsters: Option<Vec<Monster>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub room_number: u16,
    pub title: String,
    pub desc_short: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Monster {
    pub name: String,
    pub health: i16,
    pub attack: u16,
    pub defense: u16,
    pub gold: u16,
    pub desc: String,
}

impl Map {
    pub fn new(rooms: Vec<Room>) -> Self {
        Map {
            rooms,
            players: HashMap::new(),
            desc: String::new(),
        }
    }

    pub fn build(data: File) -> Result<Self, serde_json::Error> {
        info!("[MAP] Building game map...");

        let deserialized: Vec<Room> = serde_json::from_reader(&data)?;
        info!("[MAP] Game map built with {} rooms.", deserialized.len());

        Ok(Map::new(deserialized))
    }

    pub fn add_player(&mut self, player: pkt_character::Character) {
        self.players.insert(player.name.clone(), player);
    }

    pub fn player_from_name(&mut self, name: &String) -> Option<&mut pkt_character::Character> {
        self.players.get_mut(name)
    }

    pub fn player_from_stream(
        &mut self,
        stream: &Stream,
    ) -> Option<(&String, &mut pkt_character::Character)> {
        self.players.iter_mut().find(|(_, player)| {
            let author = player.author.as_ref().ok_or_else(|| false);

            match author {
                Ok(author) => Arc::ptr_eq(&author, stream),
                Err(_) => false,
            }
        })
    }

    pub fn exits(&self, room_number: u16) -> Option<Vec<&Connection>> {
        self.rooms
            .iter()
            .find(|r| r.room_number == room_number)
            .map(|room| room.connections.iter().collect())
    }

    /// Broadcast a message to all players in the game via Message packets.
    pub fn broadcast(&self, message: String) -> Result<(), std::io::Error> {
        info!("[BROADCAST] Sending message: {}", message);

        // Send the packet to the server
        for (_, player) in &self.players {
            let author = player.author.as_ref().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Author not found")
            })?;

            debug!("[BROADCAST] Sending message to {}", player.name);

            Protocol::Message(
                author.clone(),
                pkt_message::Message {
                    message_type: PktType::Message,
                    message_len: message.len() as u16,
                    recipient: player.name.clone(),
                    sender: "Server".to_string(),
                    narration: false,
                    message: message.clone(),
                },
            )
            .send()
            .unwrap_or_else(|e| {
                warn!(
                    "[BROADCAST] Failed to send message to {}: {}",
                    player.name, e
                );
            });
        }

        Ok(())
    }

    pub fn message_room(&self, room_number: u16, message: String) -> Result<(), std::io::Error> {
        info!(
            "[ROOM MESSAGE] Sending message to room {}: {}",
            room_number, message
        );

        let room = self
            .rooms
            .iter()
            .find(|r| r.room_number == room_number)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Room not found"))?;

        room.players.iter().for_each(|name| {
            let player = match self.players.get(name) {
                Some(player) => player,
                None => {
                    error!("[ROOM MESSAGE] Player '{}' doesn't exist!", name);
                    return;
                }
            };

            let author = match player.author.as_ref() {
                Some(author) => author,
                None => {
                    warn!("[ROOM MESSAGE] Player '{}' is not connected", name);
                    return;
                }
            };

            debug!("[ROOM MESSAGE] Sending message to '{}'", name);

            Protocol::Message(
                author.clone(),
                pkt_message::Message {
                    message_type: PktType::Message,
                    message_len: message.len() as u16,
                    recipient: player.name.clone(),
                    sender: "Narrator".to_string(),
                    narration: true,
                    message: message.clone(),
                },
            )
            .send()
            .unwrap_or_else(|e| {
                error!("[ROOM MESSAGE] Failed to send message to '{}': {}", name, e);
            });
        });

        Ok(())
    }

    /// Alert all players in the current room of a character change by sending a Character packet
    /// to each player in the room.
    pub fn alert_room(
        &self,
        room_number: u16,
        player: &pkt_character::Character,
    ) -> Result<(), std::io::Error> {
        info!("[ALERT] Alerting players about: '{}'", player.name);

        let room = self
            .rooms
            .iter()
            .find(|r| r.room_number == room_number)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Room not found"))?;

        room.players.iter().for_each(|name| {
            debug!("[ALERT] Alerting player: '{}'", name);

            let player = match self.players.get(name) {
                Some(player) => player,
                None => {
                    error!("[ALERT] Player '{}' doesn't exist!", name);
                    return;
                }
            };

            let author = match player.author.as_ref() {
                Some(author) => author,
                None => {
                    warn!("[ALERT] Player '{}' is not connected", player.name);
                    return;
                }
            };

            Protocol::Character(author.clone(), player.clone())
                .send()
                .unwrap_or_else(|e| {
                    error!("[ALERT] Failed to alert '{}': {}", name, e);
                })
        });

        Ok(())
    }
}
