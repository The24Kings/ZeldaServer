use serde::Serialize;
use std::io::Write;
use tracing::debug;

use crate::protocol::{
    packet::{Packet, Parser},
    pcap::PCap,
    pkt_type::PktType,
};

#[derive(Default, Serialize, Debug, Clone)]
pub struct Accept {
    pub message_type: PktType,
    pub accept_type: u8,
}

impl Accept {
    pub fn new(accept_type: PktType) -> Self {
        Accept {
            message_type: PktType::Accept,
            accept_type: accept_type.into(),
        }
    }
}

impl std::fmt::Display for Accept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Accept".to_string())
        )
    }
}

impl<'a> Parser<'a> for Accept {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.accept_type.to_le_bytes());

        // Write the packet to the buffer
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
        Ok(Accept {
            message_type: packet.message_type,
            accept_type: packet.body[0],
        })
    }
}
