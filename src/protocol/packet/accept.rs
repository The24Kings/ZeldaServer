use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct Accept {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub accept_type: u8,
}

impl<'a> Parser<'a> for Accept {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Accept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Accept {{ author: {:?}, message_type: {}, accept_type: {} }}",
            self.author, self.message_type, self.accept_type
        )
    }
}
