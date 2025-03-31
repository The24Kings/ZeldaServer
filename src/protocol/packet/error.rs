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
    pub message: Vec<u8>,
}

impl<'a> Parser<'a> for Error {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}
