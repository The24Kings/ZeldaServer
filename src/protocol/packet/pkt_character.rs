use bitflags::bitflags;
use std::io::Write;
use std::os::fd::{AsFd, AsRawFd};
use std::sync::Arc;

use crate::protocol::game::Monster;
use crate::protocol::{
    Stream,
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Debug, Clone)]
pub struct Character {
    pub author: Option<Stream>,
    pub message_type: PktType,
    pub name: Arc<str>,
    pub flags: CharacterFlags,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description_len: u16,
    pub description: Box<str>,
}

impl Character {
    pub fn to_default(incoming: &Character) -> Self {
        Character {
            author: incoming.author.clone(),
            message_type: incoming.message_type,
            name: incoming.name.clone(),
            flags: CharacterFlags::alive(),
            attack: incoming.attack,
            defense: incoming.defense,
            regen: incoming.regen,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: incoming.description_len,
            description: incoming.description.clone(),
        }
    }

    pub fn from_monster(incoming: &Monster, current_room: u16) -> Self {
        let mut flags = CharacterFlags::MONSTER;

        if incoming.health <= 0 {
            flags |= CharacterFlags::dead();
        } else {
            flags |= CharacterFlags::alive();
        };

        Character {
            author: None,
            message_type: PktType::CHARACTER,
            name: Arc::from(incoming.name.clone()),
            flags,
            attack: incoming.attack,
            defense: incoming.defense,
            regen: 0, // Monsters don't regenerate health
            health: incoming.health,
            gold: incoming.gold,
            current_room,
            description_len: incoming.desc.len() as u16,
            description: incoming.desc.clone(),
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character {
            author: None,
            message_type: PktType::CHARACTER,
            name: Arc::from("Error"),
            flags: CharacterFlags::alive(),
            attack: 0,
            defense: 0,
            regen: 0,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: 60,
            description: Box::from("Something went wrong, please close the client and try again!"),
        }
    }
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let author = match &self.author {
            Some(stream) => format!(
                "\"addr\":\"{}\",\"peer\":\"{}\",\"fd\":{}",
                stream
                    .as_ref()
                    .peer_addr()
                    .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
                stream
                    .as_ref()
                    .local_addr()
                    .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
                stream.as_fd().as_raw_fd()
            ),
            None => "None".to_string(),
        };

        write!(
            f,
            "{{\"author\":{{{}}},\"message_type\":\"{}\",\"name\":\"{}\",\"flags\":\"0b{:08b}\",\"attack\":{},\"defense\":{},\"regen\":{},\"health\":{},\"gold\":{},\"current_room\":{},\"description_len\":{},\"description\":\"{}\"}}",
            author,
            self.message_type,
            self.name,
            self.flags.bits(),
            self.attack,
            self.defense,
            self.regen,
            self.health,
            self.gold,
            self.current_room,
            self.description_len,
            self.description
        )
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CharacterFlags: u8 {
        const ALIVE = 0b10000000;
        const BATTLE = 0b01000000; // A.K.A. Join-Battle
        const MONSTER = 0b00100000;
        const STARTED = 0b00010000;
        const READY = 0b00001000;
    }
}

impl CharacterFlags {
    pub fn is_alive(&self) -> bool {
        self.contains(CharacterFlags::ALIVE)
    }

    pub fn is_battle(&self) -> bool {
        self.contains(CharacterFlags::BATTLE)
    }

    pub fn is_monster(&self) -> bool {
        self.contains(CharacterFlags::MONSTER)
    }

    pub fn is_started(&self) -> bool {
        self.contains(CharacterFlags::STARTED)
    }

    pub fn is_ready(&self) -> bool {
        self.contains(CharacterFlags::READY)
    }

    pub fn dead() -> Self {
        CharacterFlags::BATTLE | CharacterFlags::READY
    }

    pub fn alive() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE | CharacterFlags::READY
    }

    pub fn reset() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE
    }
}

impl<'a> Parser<'a> for Character {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());

        // Serialize the character name
        let mut name_bytes = self.name.as_bytes().to_vec();
        name_bytes.resize(32, 0x00); // Pad the name to 32 bytes

        packet.extend(name_bytes);

        // Serialize the flags byte
        packet.extend([self.flags.bits()]);

        // Serialize the character stats
        packet.extend(self.attack.to_le_bytes());
        packet.extend(self.defense.to_le_bytes());
        packet.extend(self.regen.to_le_bytes());
        packet.extend(self.health.to_le_bytes());
        packet.extend(self.gold.to_le_bytes());
        packet.extend(self.current_room.to_le_bytes());
        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let name = String::from_utf8_lossy(&packet.body[0..32])
            .split('\0')
            .take(1)
            .collect::<String>();
        let flags = CharacterFlags::from_bits_truncate(packet.body[32]); // Other bits are reserved for future use
        let attack = u16::from_le_bytes([packet.body[33], packet.body[34]]);
        let defense = u16::from_le_bytes([packet.body[35], packet.body[36]]);
        let regen = u16::from_le_bytes([packet.body[37], packet.body[38]]);
        let health = i16::from_le_bytes([packet.body[39], packet.body[40]]);
        let gold = u16::from_le_bytes([packet.body[41], packet.body[42]]);
        let current_room = u16::from_le_bytes([packet.body[43], packet.body[44]]);
        let description_len = u16::from_le_bytes([packet.body[45], packet.body[46]]);
        let description = String::from_utf8_lossy(&packet.body[47..]).into();

        Ok(Character {
            author: Some(packet.stream.clone()),
            message_type: packet.message_type,
            name: Arc::from(name),
            flags,
            attack,
            defense,
            regen,
            health,
            gold,
            current_room,
            description_len,
            description,
        })
    }
}
