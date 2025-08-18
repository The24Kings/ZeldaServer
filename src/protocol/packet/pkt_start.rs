use serde::Serialize;
use std::io::Write;
use tracing::debug;

use crate::protocol::{
    packet::{Packet, Parser},
    pcap::PCap,
    pkt_type::PktType,
};

#[derive(Debug, Serialize, Clone)]
pub struct Start {
    pub message_type: PktType,
}

impl Default for Start {
    fn default() -> Self {
        Start {
            message_type: PktType::START,
        }
    }
}

impl std::fmt::Display for Start {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize Start".to_string())
        )
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

        debug!("[DEBUG] Packet body:\n{}", PCap::build(packet.clone()));

        Ok(())
    }
    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Start {
            message_type: packet.message_type,
        })
    }
}
