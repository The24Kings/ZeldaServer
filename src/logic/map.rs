use lurk_lcsc::{CharacterFlags, PktCharacter, PktConnection, PktMessage, PktRoom, PktType};
use lurk_lcsc::{send_character, send_message};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, net::TcpStream, sync::Arc};
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: Box<str>,
    pub connections: HashMap<u16, Connection>,
    pub desc: Box<str>,
    pub players: Vec<Arc<str>>,
    pub monsters: Option<Vec<Monster>>,
}

impl From<Room> for PktRoom {
    fn from(room: Room) -> Self {
        PktRoom {
            packet_type: PktType::ROOM,
            room_number: room.room_number,
            room_name: room.title,
            description_len: room.desc.len() as u16,
            description: room.desc,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Connection {
    pub room_number: u16,
    pub title: Box<str>,
    pub desc_short: Box<str>,
}

impl From<Connection> for PktConnection {
    /// Create a new connection from the game map to send to the client
    fn from(conn: Connection) -> Self {
        PktConnection {
            packet_type: PktType::CONNECTION,
            room_number: conn.room_number,
            room_name: conn.title,
            description_len: conn.desc_short.len() as u16,
            description: conn.desc_short,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Monster {
    pub name: Arc<str>,
    pub current_room: u16,
    pub health: i16,
    pub attack: u16,
    pub defense: u16,
    pub gold: u16,
    pub desc: Box<str>,
}

impl From<Monster> for PktCharacter {
    fn from(monster: Monster) -> Self {
        let mut flags = CharacterFlags::MONSTER | CharacterFlags::BATTLE;

        if monster.health <= 0 {
            flags |= CharacterFlags::dead();
        } else {
            flags |= CharacterFlags::alive();
        };

        Self {
            author: None,
            packet_type: PktType::CHARACTER,
            name: monster.name.clone(),
            flags,
            attack: monster.attack,
            defense: monster.defense,
            regen: 0,
            health: monster.health,
            gold: monster.gold,
            current_room: monster.current_room,
            description_len: monster.desc.len() as u16,
            description: monster.desc.clone(),
        }
    }
}

impl From<&Monster> for PktCharacter {
    fn from(monster: &Monster) -> Self {
        let mut flags = CharacterFlags::MONSTER | CharacterFlags::BATTLE;

        if monster.health <= 0 {
            flags |= CharacterFlags::dead();
        } else {
            flags |= CharacterFlags::alive();
        };

        Self {
            author: None,
            packet_type: PktType::CHARACTER,
            name: monster.name.clone(),
            flags,
            attack: monster.attack,
            defense: monster.defense,
            regen: 0,
            health: monster.health,
            gold: monster.gold,
            current_room: monster.current_room,
            description_len: monster.desc.len() as u16,
            description: monster.desc.clone(),
        }
    }
}

impl From<&mut Monster> for PktCharacter {
    fn from(monster: &mut Monster) -> Self {
        let mut flags = CharacterFlags::MONSTER | CharacterFlags::BATTLE;

        if monster.health <= 0 {
            flags |= CharacterFlags::dead();
        } else {
            flags |= CharacterFlags::alive();
        };

        Self {
            author: None,
            packet_type: PktType::CHARACTER,
            name: monster.name.clone(),
            flags,
            attack: monster.attack,
            defense: monster.defense,
            regen: 0,
            health: monster.health,
            gold: monster.gold,
            current_room: monster.current_room,
            description_len: monster.desc.len() as u16,
            description: monster.desc.clone(),
        }
    }
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
    players: &mut HashMap<Arc<str>, PktCharacter>,
    stream: Arc<TcpStream>,
) -> Option<(&Arc<str>, &mut PktCharacter)> {
    players.iter_mut().find(|(_, player)| {
        player
            .author
            .as_ref()
            .is_some_and(|author| Arc::ptr_eq(author, &stream))
    })
}

/// Broadcast a message to all players in the game via Message packets.
pub fn broadcast(players: &HashMap<Arc<str>, PktCharacter>, message: String) {
    info!("[BROADCAST] Sending message: {}", message);

    // Send the packet to the server
    for (name, player) in players {
        let author = match player.author.as_ref() {
            Some(author) => author,
            None => continue,
        };

        debug!("[BROADCAST] Sending message to {}", name);

        send_message!(author.clone(), PktMessage::server(name, &message));
    }
}

pub fn message_room(
    players: &HashMap<Arc<str>, PktCharacter>,
    room: &Room,
    message: String,
    narration: bool,
) {
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
            PktMessage::narrator(&player.name, &message)
        } else {
            PktMessage::server(&player.name, &message)
        };

        send_message!(author.clone(), message);
    });
}

/// Alert all players in the current room of a character change by sending a Character packet
/// to each player in the room.
pub fn alert_room(players: &HashMap<Arc<str>, PktCharacter>, room: &Room, alert: &PktCharacter) {
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

        send_character!(author.clone(), alert.clone());
    });
}
