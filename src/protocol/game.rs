use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, sync::Arc};
use tracing::{debug, error, info, warn};

use crate::protocol::{
    Protocol, Stream,
    packet::{pkt_character, pkt_message},
    pkt_type::PktType,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: String,
    pub connections: HashMap<u16, Connection>,
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

pub fn build(data: File) -> Result<HashMap<u16, Room>, serde_json::Error> {
    let mut rooms: HashMap<u16, Room> = HashMap::new();

    info!("[MAP] Building game map...");

    let deserialized: Vec<Room> = serde_json::from_reader(&data)?;
    info!("[MAP] Game map built with {} rooms.", deserialized.len());

    for room in deserialized {
        rooms.insert(room.room_number, room);
    }

    Ok(rooms)
}

pub fn exits(rooms: &HashMap<u16, Room>, room_number: u16) -> Option<HashMap<u16, Connection>> {
    rooms.get(&room_number).map(|room| room.connections.clone())
}

pub fn add_player(
    players: &mut HashMap<String, pkt_character::Character>,
    player: pkt_character::Character,
) {
    players.insert(player.name.clone(), player);
}

pub fn player_from_stream(
    players: &mut HashMap<String, pkt_character::Character>,
    stream: Stream,
) -> Option<(&String, &mut pkt_character::Character)> {
    players.iter_mut().find(|(_, player)| {
        player
            .author
            .as_ref()
            .map_or(false, |author| Arc::ptr_eq(&author, &stream))
    })
}

/// Broadcast a message to all players in the game via Message packets.
pub fn broadcast(
    players: &HashMap<String, pkt_character::Character>,
    message: String,
) -> Result<(), std::io::Error> {
    info!("[BROADCAST] Sending message: {}", message);

    // Send the packet to the server
    for (_, player) in players {
        let author = player
            .author
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Author not found"))?;

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

pub fn message_room(
    players: &HashMap<String, pkt_character::Character>,
    rooms: &HashMap<u16, Room>,
    room_number: u16,
    message: String,
) -> Result<(), std::io::Error> {
    info!(
        "[ROOM MESSAGE] Sending message to room {}: {}",
        room_number, message
    );

    let room = rooms
        .get(&room_number)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Room not found"))?;

    room.players.iter().for_each(|name| {
        let player = match players.get(name) {
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
    players: &HashMap<String, pkt_character::Character>,
    rooms: &HashMap<u16, Room>,
    room_number: u16,
    alert: &pkt_character::Character,
) -> Result<(), std::io::Error> {
    info!("[ALERT] Alerting players about: '{}'", alert.name);

    let room = rooms
        .get(&room_number)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Room not found"))?;

    room.players.iter().for_each(|name| {
        debug!("[ALERT] Alerting player: '{}'", name);

        let player = match players.get(name) {
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

        Protocol::Character(author.clone(), alert.clone())
            .send()
            .unwrap_or_else(|e| {
                error!("[ALERT] Failed to alert '{}': {}", name, e);
            })
    });

    Ok(())
}
