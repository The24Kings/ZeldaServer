use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct Version {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub major_rev: u8,
    pub minor_rev: u8,
    pub extension_len: u16, // Can be 0, just ignore
    pub extensions: Vec<u8>, // 0-1 length, 2-+ first extention;
}

impl<'a> Parser<'a> for Version {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        Ok(Version {
            author: packet.author,
            message_type: packet.message_type,
            major_rev: packet.body[0],
            minor_rev: packet.body[1],
            extension_len: 0,       //TODO: LurkCat doesn't send this, add it in the future
            extensions: Vec::new(), //TODO: LurkCat doesn't send this, add it in the future
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Version {{ author: {:?}, message_type: {}, major_rev: {}, minor_rev: {}, extension_len: {}, extensions: {:?} }}",
            self.author, self.message_type, self.major_rev, self.minor_rev, self.extension_len, self.extensions
        )
    }
}