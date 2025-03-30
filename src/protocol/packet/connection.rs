use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct Connection {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub room_number: u16,
    pub room_name: Vec<u8>,
    pub description_len: u16,
    pub description: Vec<u8>,
}

impl<'a> Parser<'a> for Connection {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}