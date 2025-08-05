use serde::Serialize;
use std::io::Write;
use tracing::debug;

use crate::protocol::{
    packet::{Packet, Parser},
    pkt_type::PktType,
};

#[derive(Default, Serialize, Debug, Clone)]
pub struct Version {
    pub message_type: PktType,
    pub major_rev: u8,
    pub minor_rev: u8,
    pub extension_len: u16,
    pub extensions: Option<Vec<u8>>, // 0-1 length, 2+ extension;
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Version".to_string())
        )
    }
}

impl<'a> Parser<'a> for Version {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.major_rev.to_le_bytes());
        packet.extend(self.minor_rev.to_le_bytes());
        packet.extend(self.extension_len.to_le_bytes());

        if let Some(extensions) = &self.extensions {
            packet.extend(extensions);
        }

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
        Ok(Version {
            message_type: packet.message_type,
            major_rev: packet.body[0],
            minor_rev: packet.body[1],
            extension_len: 0,
            extensions: None, // Server currently does not use extensions
        })
    }
}
