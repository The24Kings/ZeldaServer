use serde::{Deserialize, Serialize};
use std::{env, fs::File};
use tracing::{error, info, warn};

use crate::protocol::{Protocol, packet::pkt_character, packet::pkt_message, pkt_type::PktType};

#[derive(Debug)]
pub struct Map {
    pub init_points: u16,
    pub stat_limit: u16,
    pub rooms: Vec<Room>,
    pub players: Vec<pkt_character::Character>,
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: String,
    pub connections: Vec<Connection>,
    pub desc: String,
    pub players: Vec<usize>, // Indices of players in the map's player list
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
            init_points: env::var("INITIAL_POINTS")
                .expect("[MAP] INITIAL_POINTS must be set.")
                .parse()
                .expect("[MAP] Failed to parse INITIAL_POINTS"),
            stat_limit: env::var("STAT_LIMIT")
                .expect("[MAP] STAT_LIMIT must be set.")
                .parse()
                .expect("[MAP] Failed to parse STAT_LIMIT"),
            rooms,
            players: Vec::new(),
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
        self.players.push(player);
    }

    pub fn get_exits(&self, id: u16) -> Option<Vec<&Connection>> {
        self.rooms
            .iter()
            .find(|r| r.room_number == id)
            .map(|room| room.connections.iter().collect())
    }

    /// Broadcast a message to all players in the game via Message packets.
    pub fn broadcast(&self, message: String) -> Result<(), std::io::Error> {
        info!("[BROADCAST] Sending message: {}", message);

        // Send the packet to the server
        for player in &self.players {
            let author = player.author.as_ref().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Author not found")
            })?;

            info!("[BROADCAST] Sending message to {}", player.name);

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

    /// Alert all players in the current room of a character change by sending a Character packet
    /// to each player in the room.
    pub fn alert_room(
        &self,
        room_number: u16,
        player: &pkt_character::Character,
    ) -> Result<(), std::io::Error> {
        info!("[ALERT] Alerting players about: {}", player.name);

        let author = player
            .author
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Author not found"))?;

        let room = self
            .rooms
            .iter()
            .find(|r| r.room_number == room_number)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Room not found"))?;

        room.players
            .iter()
            .for_each(|&player_index| match self.players.get(player_index) {
                Some(to_alert) => Protocol::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[ALERT] Failed to alert {}: {}", to_alert.name, e);
                    }),
                None => {
                    error!("[ALERT] Invalid player index: {}", player_index);
                }
            });

        Ok(())
    }
}
