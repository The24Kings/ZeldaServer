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
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Error {{ author: {:?}, message_type: {}, error: {}, message_len: {}, message: {} }}",
            self.author, self.message_type, self.error, self.message_len, self.message
        )
    }
}