use std::io::Write;
use std::os::fd::AsRawFd;
use std::{fmt::LowerHex, os::fd::AsFd};
use tracing::debug;

use crate::protocol::map::Monster;
use crate::protocol::{
    Stream,
    packet::{Packet, Parser},
    pcap::PCap,
    pkt_type::PktType,
};

#[derive(Debug, Clone)]
pub struct Character {
    pub author: Option<Stream>,
    pub message_type: PktType,
    pub name: String,
    pub flags: CharacterFlags,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description_len: u16,
    pub description: String,
}

impl Character {
    pub fn from(author: Option<Stream>, incoming: &Character) -> Self {
        // TODO: Look into, it might be redundant now, since the Character packet holds the author...
        Character {
            author,
            message_type: incoming.message_type,
            name: incoming.name.clone(),
            flags: CharacterFlags::default(),
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
        Character {
            author: None,
            message_type: PktType::Character,
            name: incoming.name.clone(),
            flags: CharacterFlags::activate(true),
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
            message_type: PktType::Character,
            name: "Error".to_string(),
            flags: CharacterFlags::default(),
            attack: 0,
            defense: 0,
            regen: 0,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: 60,
            description: "Something went wrong, please close the client and try again!".to_string(),
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
            "{{\"author\":{{{}}},\"message_type\":\"{}\",\"name\":\"{}\",\"flags\":{{\"alive\":{},\"battle\":{},\"monster\":{},\"started\":{},\"ready\":{}}},\"attack\":{},\"defense\":{},\"regen\":{},\"health\":{},\"gold\":{},\"current_room\":{},\"description_len\":{},\"description\":\"{}\"}}",
            author,
            self.message_type,
            self.name,
            self.flags.alive,
            self.flags.battle,
            self.flags.monster,
            self.flags.started,
            self.flags.ready,
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

#[derive(Debug, Clone)]
pub struct CharacterFlags {
    pub alive: bool,
    pub battle: bool, // A.K.A. Join-Battle
    pub monster: bool,
    pub started: bool,
    pub ready: bool,
}

impl Default for CharacterFlags {
    fn default() -> Self {
        CharacterFlags {
            alive: true,
            battle: true,
            monster: false,
            started: false,
            ready: true,
        }
    }
}

impl LowerHex for CharacterFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}",
            (self.alive as u8) << 7
                | (self.battle as u8) << 6
                | (self.monster as u8) << 5
                | (self.started as u8) << 4
                | (self.ready as u8) << 3
        )
    }
}

impl CharacterFlags {
    pub fn deactivate(monster: bool) -> Self {
        CharacterFlags {
            alive: false,
            battle: false,
            monster,
            started: false,
            ready: false,
        }
    }

    pub fn activate(monster: bool) -> Self {
        CharacterFlags {
            alive: true,
            battle: true,
            monster,
            started: false,
            ready: true,
        }
    }

    pub fn dead(monster: bool) -> Self {
        CharacterFlags {
            alive: false,
            battle: false,
            monster,
            started: false,
            ready: true,
        }
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

        // Serialize the flags byte in little-endian order
        let mut flags: u8 = 0x00;

        flags |= if self.flags.alive { 0b10000000 } else { 0x00 };
        flags |= if self.flags.battle { 0b01000000 } else { 0x00 };
        flags |= if self.flags.monster { 0b00100000 } else { 0x00 };
        flags |= if self.flags.started { 0b00010000 } else { 0x00 };
        flags |= if self.flags.ready { 0b00001000 } else { 0x00 };

        packet.extend(flags.to_le_bytes());

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

        debug!("[DEBUG] Packet body:\n{}", PCap::build(packet.clone()));

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let name = String::from_utf8_lossy(&packet.body[0..32])
            .split('\0')
            .take(1)
            .collect::<String>();
        let flags = packet.body[32];
        let attack = u16::from_le_bytes([packet.body[33], packet.body[34]]);
        let defense = u16::from_le_bytes([packet.body[35], packet.body[36]]);
        let regen = u16::from_le_bytes([packet.body[37], packet.body[38]]);
        let health = i16::from_le_bytes([packet.body[39], packet.body[40]]);
        let gold = u16::from_le_bytes([packet.body[41], packet.body[42]]);
        let current_room = u16::from_le_bytes([packet.body[43], packet.body[44]]);
        let description_len = u16::from_le_bytes([packet.body[45], packet.body[46]]);
        let description = String::from_utf8_lossy(&packet.body[47..]).to_string();

        // Parse the flags byte in little-endian order
        let flags = CharacterFlags {
            alive: flags & 0b10000000 != 0,
            battle: flags & 0b01000000 != 0,
            monster: flags & 0b00100000 != 0,
            started: flags & 0b00010000 != 0,
            ready: flags & 0b00001000 != 0,
        }; // Other bits are reserved for future use

        Ok(Character {
            author: Some(packet.stream.clone()),
            message_type: packet.message_type,
            name,
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
