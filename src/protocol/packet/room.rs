use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Room {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub room_number: u16, // Same as room_num in ChangeRoom
    pub room_name: String,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Room {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }
    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Room::default())
    }
}

impl std::fmt::Display for Room {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Room {{ author: {:?}, message_type: {}, room_number: {}, room_name: {}, description_len: {}, description: {} }}",
            self.author,
            self.message_type,
            self.room_number,
            self.room_name,
            self.description_len,
            self.description
        )
    }
}