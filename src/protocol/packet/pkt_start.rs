use crate::{
    debug_packet,
    protocol::{
        packet::{Packet, Parser},
        pkt_type::PktType,
    },
};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Start {
    pub message_type: PktType,
}

impl Default for Start {
    fn default() -> Self {
        Start {
            message_type: PktType::Start,
        }
    }
}

impl<'a> Parser<'a> for Start {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());

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
        Ok(Start {
            message_type: packet.message_type,
        })
    }
}
