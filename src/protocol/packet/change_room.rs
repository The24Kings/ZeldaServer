use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct ChangeRoom {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub room_num: u16
}

impl<'a> Parser<'a> for ChangeRoom {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for ChangeRoom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChangeRoom {{ author: {:?}, message_type: {}, room_num: {} }}",
            self.author, self.message_type, self.room_num
        )
    }
}