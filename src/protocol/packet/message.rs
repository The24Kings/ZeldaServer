use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct Message {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub message_len: u16,
    pub recipient: String,
    pub sender: String,
    pub message: String,
}

impl<'a> Parser<'a> for Message {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(Message {
            author: packet.author,
            message_type: packet.message_type,
            message_len: 0,
            recipient: String::new(),
            sender: String::new(),
            message: String::new(),
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
