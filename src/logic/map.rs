use indexmap::IndexSet;
use lurk_lcsc::{CharacterFlags, PktCharacter, PktConnection, PktRoom, PktType};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, net::TcpStream, sync::Arc};
use tracing::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_number: u16,
    pub title: Box<str>,
    pub connections: HashMap<u16, Connection>,
    pub desc: Box<str>,
    pub players: IndexSet<Arc<str>>,
    pub monsters: Option<Vec<Monster>>,
}

impl From<&Room> for PktRoom {
    fn from(room: &Room) -> Self {
        PktRoom {
            packet_type: PktType::ROOM,
            room_number: room.room_number,
            room_name: room.title.clone(),
            description_len: room.desc.len() as u16,
            description: room.desc.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Connection {
    pub room_number: u16,
    pub title: Box<str>,
    pub desc_short: Box<str>,
}

impl From<&Connection> for PktConnection {
    fn from(conn: &Connection) -> Self {
        PktConnection {
            packet_type: PktType::CONNECTION,
            room_number: conn.room_number,
            room_name: conn.title.clone(),
            description_len: conn.desc_short.len() as u16,
            description: conn.desc_short.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Monster {
    pub name: Arc<str>,
    pub current_room: u16,
    pub max_health: i16,
    pub health: i16,
    pub attack: u16,
    pub defense: u16,
    pub gold: u16,
    pub desc: Box<str>,
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

    info!("Building game map...");

    let deserialized: Vec<Room> = serde_json::from_reader(&data)?;
    info!("Game map built with {} rooms.", deserialized.len());

    for room in deserialized {
        rooms.insert(room.room_number, room);
    }

    Ok(rooms)
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
