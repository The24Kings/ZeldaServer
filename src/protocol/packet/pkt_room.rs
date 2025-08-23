use serde::Serialize;
use std::io::Write;

use crate::protocol::{
    game,
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Default, Serialize, Debug, Clone)]
pub struct Room {
    pub message_type: PktType,
    pub room_number: u16, // Same as room_number in ChangeRoom
    pub room_name: String,
    pub description_len: u16,
    pub description: String,
}

impl std::fmt::Display for Room {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Room".to_string())
        )
    }
}

impl Room {
    /// Create a new room for the game map (Not to be confused with the Room packet sent to the client)
    pub fn new(room: u16, title: String, desc: String) -> Self {
        Room {
            message_type: PktType::ROOM,
            room_number: room,
            room_name: title,
            description_len: desc.len() as u16,
            description: desc,
        }
    }
}

impl From<game::Room> for Room {
    fn from(room: game::Room) -> Self {
        Room {
            message_type: PktType::ROOM,
            room_number: room.room_number,
            room_name: room.title.clone(),
            description_len: room.desc.len() as u16,
            description: room.desc.clone(),
        }
    }
}

impl<'a> Parser<'a> for Room {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.room_number.to_le_bytes());

        let mut room_name_bytes = self.room_name.as_bytes().to_vec();
        room_name_bytes.resize(32, 0); // Pad with zeros to 32 bytes
        packet.extend(room_name_bytes);

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
        let message_type = packet.message_type;
        let room_number = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let room_name = String::from_utf8_lossy(&packet.body[2..34])
            .trim_end_matches('\0')
            .to_string();
        let description_len = u16::from_le_bytes([packet.body[34], packet.body[35]]);
        let description = String::from_utf8_lossy(&packet.body[36..]).to_string();

        Ok(Room {
            message_type,
            room_number,
            room_name,
            description_len,
            description,
        })
    }
}
