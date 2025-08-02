use std::io::Write;

use crate::{
    debug_packet,
    protocol::{
        packet::{Packet, Parser},
        pkt_type::PktType,
    },
};

#[derive(Default, Debug, Clone)]
pub struct Game {
    pub message_type: PktType,
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Game {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
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

        debug_packet!(&packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        println!("[GAME] Deserializing packet: {}", packet);

        let initial_points = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let stat_limit = u16::from_le_bytes([packet.body[2], packet.body[3]]);
        let description_len = u16::from_le_bytes([packet.body[4], packet.body[5]]);

        Ok(Game {
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
