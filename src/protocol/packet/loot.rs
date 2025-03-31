use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Loot {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub target_name: String,
}

impl<'a> Parser<'a> for Loot {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Loot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n  Loot {{
            \n    message_type: {},
            \n    target_name: {}
            \n  }}",
            self.message_type, self.target_name
        )
    }
}
