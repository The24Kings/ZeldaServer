use std::io::Write;

use crate::{debug_packet, protocol::packet::{Packet, Parser}};

#[derive(Debug, Clone)]
pub struct Fight {
    pub message_type: u8,
}

impl Default for Fight {
    fn default() -> Self {
        Fight {
            message_type: 3,
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

        debug_packet!(&packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Fight {
            message_type: packet.message_type,
        })
    }
}
