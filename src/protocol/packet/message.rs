use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Message {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub message_len: u16,
    pub recipient: String,
    pub sender: String,
    pub narration: bool,
    pub message: String,
}

impl<'a> Parser<'a> for Message {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        let message_len = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Process the names for recipient and sender
        let mut r_bytes = packet.body[2..34].to_vec();
        let mut s_bytes = packet.body[34..66].to_vec();

        // If the last 2 bytes of the sender is 0x00 0x01, it means the sender is a narrator
        let narration = match s_bytes.get(32..34) {
            Some(&[0x00, 0x01]) => {
                s_bytes.truncate(32); // Remove the last 2 bytes
                true
            }
            _ => false,
        };

        let sender = String::from_utf8_lossy(&s_bytes).trim_end_matches('\0').to_string();
        let recipient = String::from_utf8_lossy(&r_bytes).trim_end_matches('\0').to_string();
        let message = String::from_utf8_lossy(&packet.body[66..]).to_string();

        Ok(Message {
            author: packet.author,
            message_type: packet.message_type,
            message_len,
            recipient,
            sender,
            narration,
            message,
        })
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Message {{ author: {:?}, message_type: {}, message_len: {:?}, recipient: {:?}, sender: {:?}, message: {:?} }}",
            self.author,
            self.message_type,
            self.message_len,
            self.recipient,
            self.sender,
            self.message
        )
    }
}
