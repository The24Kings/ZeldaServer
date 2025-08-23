use serde::Serialize;
use std::io::Write;

use crate::protocol::{
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Debug, Serialize, Clone)]
pub struct Fight {
    pub message_type: PktType,
}

impl Default for Fight {
    fn default() -> Self {
        Fight {
            message_type: PktType::FIGHT,
        }
    }
}

impl std::fmt::Display for Fight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Fight".to_string())
        )
    }
}

impl<'a> Parser<'a> for Fight {
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

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Fight {
            message_type: packet.message_type,
        })
    }
}
