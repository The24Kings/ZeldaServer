use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Version {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub major_rev: u8,
    pub minor_rev: u8,
    pub extension_len: u16,
    pub extensions: Option<Vec<u8>>, // 0-1 length, 2+ extention;
}

impl<'a> Parser<'a> for Version {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
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

        println!("[VERSION] Serialized packet: {:?}", packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Version {
            author: packet.author,
            message_type: packet.message_type,
            major_rev: packet.body[0],
            minor_rev: packet.body[1],
            extension_len: 0,
            extensions: None, // Server currently does not use extensions
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Version {{ author: {:?}, message_type: {}, major_rev: {}, minor_rev: {}, extension_len: {}, extensions: {:?} }}",
            self.author,
            self.message_type,
            self.major_rev,
            self.minor_rev,
            self.extension_len,
            self.extensions
        )
    }
}
