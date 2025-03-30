use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct PVPFight {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub target_name: String,
}

impl<'a> Parser<'a> for PVPFight {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        // Implement serialization logic here
        Ok(())
    }
    fn deserialize(_packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(PVPFight::default())
    }
}