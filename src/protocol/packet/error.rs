use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::error::ErrorCode;
use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Error {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub error: ErrorCode,
    pub message_len: u16,
    pub message: String,
}

impl<'a> Parser<'a> for Error {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        packet.push(self.error.clone().into());
        packet.extend(self.message_len.to_le_bytes());
        packet.extend(self.message.as_bytes());

        // Send the packet to the author
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        println!("[ERROR] Serialized packet: {}",
            packet
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let message_type = packet.message_type;
        let error = ErrorCode::from(packet.body[1]);
        let message_len = u16::from_le_bytes([packet.body[2], packet.body[3]]);
        let message = String::from_utf8_lossy(&packet.body[4..])
            .trim_end_matches('\0')
            .to_string();
        
        Ok(Error {
            author: packet.author,
            message_type,
            error,
            message_len,
            message,
        })
    }
}
