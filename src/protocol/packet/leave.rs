use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Debug, Clone)]
pub struct Leave {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
}

impl Default for Leave {
    fn default() -> Self {
        Leave {
            author: None,
            message_type: 12
        }
    }
}

impl<'a> Parser<'a> for Leave {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[LEAVE] Serialized packet: {}",
            packet
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );
        
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Leave {
            author: packet.author,
            message_type: packet.message_type,
        })
    }
}
