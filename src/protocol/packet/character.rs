use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Character {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub name: String,
    pub flags: u8,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Character {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Character {{ author: {:?}, message_type: {}, name: {}, flags: {}, attack: {}, defense: {}, regen: {}, health: {}, gold: {}, current_room: {}, description_len: {}, description: {} }}",
            self.author,
            self.message_type,
            self.name,
            format!("{:08b}", self.flags),
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
