use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, sync::Arc};
use tracing::{debug, error, info, warn};

use crate::protocol::{
    Protocol, Stream,
    packet::{pkt_character, pkt_message},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: String,
    pub connections: HashMap<u16, Connection>,
    pub desc: String,
    pub players: Vec<Arc<str>>,
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
    pub name: Arc<str>,
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

pub fn player_from_stream(
    players: &mut HashMap<Arc<str>, pkt_character::Character>,
    stream: Stream,
) -> Option<(&Arc<str>, &mut pkt_character::Character)> {
    players.iter_mut().find(|(_, player)| {
        player
            .author
            .as_ref()
            .map_or(false, |author| Arc::ptr_eq(&author, &stream))
    })
}

/// Broadcast a message to all players in the game via Message packets.
pub fn broadcast(
    players: &HashMap<Arc<str>, pkt_character::Character>,
    message: String,
) -> Result<(), std::io::Error> {
    info!("[BROADCAST] Sending message: {}", message);

    // Send the packet to the server
    for (name, player) in players {
        let author = match player.author.as_ref() {
            Some(author) => author,
            None => continue,
        };

        debug!("[BROADCAST] Sending message to {}", name);

        Protocol::Message(
            author.clone(),
            pkt_message::Message::server(&name, &message),
        )
        .send()
        .unwrap_or_else(|e| {
            warn!("[BROADCAST] Failed to send message to {}: {}", name, e);
        });
    }

    Ok(())
}

pub fn message_room(
    players: &HashMap<Arc<str>, pkt_character::Character>,
    room: &Room,
    message: String,
    narration: bool,
) -> Result<(), std::io::Error> {
    info!(
        "[ROOM MESSAGE] Messaging room {}: {}",
        room.room_number, message
    );

    room.players.iter().for_each(|name| {
        let player = match players.get(name) {
            Some(player) => player,
            None => return,
        };

        let author = match player.author.as_ref() {
            Some(author) => author,
            None => return,
        };

        debug!("[ROOM MESSAGE] Sending message to '{}'", name);

        let message = if narration {
            pkt_message::Message::narrator(&player.name, &message)
        } else {
            pkt_message::Message::server(&player.name, &message)
        };

        Protocol::Message(author.clone(), message)
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
    players: &HashMap<Arc<str>, pkt_character::Character>,
    room: &Room,
    alert: &pkt_character::Character,
) -> Result<(), std::io::Error> {
    info!("[ALERT] Alerting players about: '{}'", alert.name);

    room.players.iter().for_each(|name| {
        debug!("[ALERT] Alerting player: '{}'", name);

        let player = match players.get(name) {
            Some(player) => player,
            None => return,
        };

        let author = match player.author.as_ref() {
            Some(author) => author,
            None => return,
        };

        Protocol::Character(author.clone(), alert.clone())
            .send()
            .unwrap_or_else(|e| {
                error!("[ALERT] Failed to alert '{}': {}", name, e);
            })
    });

    Ok(())
}
