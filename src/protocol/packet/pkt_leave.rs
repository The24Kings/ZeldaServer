use serde::Serialize;
use std::io::Write;
use tracing::debug;

use crate::protocol::{
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Debug, Serialize, Clone)]
pub struct Leave {
    pub message_type: PktType,
}

impl Default for Leave {
    fn default() -> Self {
        Leave {
            message_type: PktType::Leave,
        }
    }
}

impl std::fmt::Display for Leave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Leave".to_string())
        )
    }
}

impl<'a> Parser<'a> for Leave {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        debug!("{:?}", packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Leave {
            message_type: packet.message_type,
        })
    }
}
