use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Game {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Game {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        packet.extend(self.initial_points.to_le_bytes());
        packet.extend(self.stat_limit.to_le_bytes());
        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[GAME] Serialized packet: {:?}", packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let initial_points = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let stat_limit = u16::from_le_bytes([packet.body[2], packet.body[3]]);
        let description_len = u16::from_le_bytes([packet.body[4], packet.body[5]]);

        Ok(Game {
            author: packet.author,
            message_type: packet.message_type,
            initial_points,
            stat_limit,
            description_len,
            description: String::from_utf8(packet.body[6..(6 + description_len as usize)].to_vec())
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to parse description: {}", e),
                    )
                })?,
        })
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game {{ initial_points: {}, stat_limit: {}, description_len: {}, description: {} }}",
            self.initial_points, self.stat_limit, self.description_len, self.description
        )
    }
}
