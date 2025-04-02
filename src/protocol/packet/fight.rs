use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Debug, Clone)]
pub struct Fight {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
}

impl Default for Fight {
    fn default() -> Self {
        Fight {
            author: None,
            message_type: 3
        }
    }
}

impl<'a> Parser<'a> for Fight {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        
        // Send the packet to the author
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[FIGHT] Serialized packet: {}",
            packet
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Fight {
            author: packet.author,
            message_type: packet.message_type,
        })
    }
}
