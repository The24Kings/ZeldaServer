use serde::Serialize;
use std::io::Write;

use crate::protocol::{
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Serialize)]
pub struct PktConnection {
    pub message_type: PktType,
    pub room_number: u16,
    pub room_name: Box<str>,
    pub description_len: u16,
    pub description: Box<str>,
}

impl std::fmt::Display for PktConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Connection".to_string())
        )
    }
}

impl<'a> Parser<'a> for PktConnection {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.room_number.to_le_bytes());

        let mut room_name_bytes = self.room_name.as_bytes().to_vec();
        room_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(room_name_bytes);

        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Send the packet to the author
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
            .into();
        let description_len = u16::from_le_bytes([packet.body[34], packet.body[35]]);
        let description = String::from_utf8_lossy(&packet.body[36..]).into();

        Ok(PktConnection {
            message_type,
            room_number,
            room_name,
            description_len,
            description,
        })
    }
}
