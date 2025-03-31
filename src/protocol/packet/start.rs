use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Start {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
}

impl<'a> Parser<'a> for Start {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }
    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Start::default())
    }
}

impl std::fmt::Display for Start {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "\n  Start {{
            \n    message_type: {}
            \n  }}", 
            self.message_type
        )
    }
}