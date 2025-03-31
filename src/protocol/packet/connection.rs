use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Connection {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub room_number: u16,
    pub room_name: String,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Connection {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Connection {{ author: {:?}, message_type: {}, room_number: {}, room_name: {}, description_len: {}, description: {} }}",
            self.author,
            self.message_type,
            self.room_number,
            self.room_name,
            self.description_len,
            self.description
        )
    }
}