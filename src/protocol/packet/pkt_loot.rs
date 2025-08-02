use std::io::Write;

use crate::{
    debug_packet,
    protocol::{
        packet::{Packet, Parser},
        pkt_type::PktType,
    },
};

#[derive(Default, Debug, Clone)]
pub struct Loot {
    pub message_type: PktType,
    pub target_name: String,
}

impl<'a> Parser<'a> for Loot {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());

        let mut target_name_bytes = self.target_name.as_bytes().to_vec();
        target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(target_name_bytes);

        // Send the packet to the author
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
        let message_type = packet.message_type;
        let target_name = String::from_utf8_lossy(&packet.body[0..32])
            .trim_end_matches('\0')
            .to_string();

        Ok(Loot {
            message_type,
            target_name,
        })
    }
}
