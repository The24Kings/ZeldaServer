use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Fight {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
}

impl<'a> Parser<'a> for Fight {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}