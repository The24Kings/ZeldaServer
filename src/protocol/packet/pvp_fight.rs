use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct PVPFight {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub target_name: String,
}

impl<'a> Parser<'a> for PVPFight {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);

        let mut target_name_bytes = self.target_name.as_bytes().to_vec();
        target_name_bytes.resize(32, 0x00); // Pad the name to 32 bytes
        packet.extend(target_name_bytes);

        // Send the packet to the author
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[PVPFIGHT] Serialized packet: {}",
            packet
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        Ok(())
    }
    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let author = packet.author.clone();
        let message_type = packet.message_type;
        let target_name = String::from_utf8_lossy(&packet.body[0..32]).trim_end_matches('\0').to_string();
        
        Ok(PVPFight {
            author,
            message_type,
            target_name,
        })
    }
}
