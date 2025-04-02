use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct ChangeRoom {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub room_num: u16
}

impl<'a> Parser<'a> for ChangeRoom {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        packet.extend(self.room_num.to_le_bytes());

        // Write the packet to the buffer
        _writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[CHANGE_ROOM] Serialized packet: {}",
            packet
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );
        
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        println!("[CHANGE_ROOM] Deserializing packet: {}", packet);

        let room_num = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Implement deserialization logic here
        Ok(ChangeRoom { 
            author: packet.author, 
            message_type: packet.message_type,
            room_num
        })
    }
}
