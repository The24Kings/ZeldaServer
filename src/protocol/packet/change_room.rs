use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct ChangeRoom {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub room_num: u16
}

impl<'a> Parser<'a> for ChangeRoom {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}
