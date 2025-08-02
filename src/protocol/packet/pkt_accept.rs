use crate::{
    debug_packet,
    protocol::{
        packet::{Packet, Parser},
        pkt_type::PktType,
    },
};
use std::io::Write;

#[derive(Default, Debug, Clone)]
pub struct Accept {
    pub message_type: PktType,
    pub accept_type: u8,
}

impl Accept {
    pub fn new(accept_type: u8) -> Self {
        Accept {
            message_type: PktType::Accept,
            accept_type,
        }
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

        debug_packet!(&packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        println!("[ACCEPT] Deserializing packet: {}", packet);

        Ok(Accept {
            message_type: packet.message_type,
            accept_type: packet.body[0],
        })
    }
}
