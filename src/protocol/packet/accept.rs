use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::{debug_packet, protocol::packet::{Packet, Parser}};

#[derive(Default, Debug, Clone)]
pub struct Accept {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub accept_type: u8,
}

impl Accept {
    pub fn new(author: Option<Arc<TcpStream>>, accept_type: u8) -> Self {
        Accept {
            author,
            message_type: 8,
            accept_type,
        }
    }
}

impl<'a> Parser<'a> for Accept {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        packet.extend(self.accept_type.to_le_bytes());

        // Write the packet to the buffer
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
        println!("[ACCEPT] Deserializing packet: {}", packet);

        Ok(Accept {
            author: packet.author,
            message_type: packet.message_type,
            accept_type: packet.body[0],
        })
    }
}
