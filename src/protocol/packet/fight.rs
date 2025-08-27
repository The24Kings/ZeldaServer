use serde::Serialize;
use std::io::Write;

use crate::protocol::{
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Serialize)]
pub struct PktFight {
    pub message_type: PktType,
}

impl Default for PktFight {
    fn default() -> Self {
        PktFight {
            message_type: PktType::FIGHT,
        }
    }
}

impl std::fmt::Display for PktFight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Fight".to_string())
        )
    }
}

impl<'a> Parser<'a> for PktFight {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
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

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(PktFight {
            message_type: packet.message_type,
        })
    }
}
