use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Debug, Clone)]
pub struct Leave {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
}

impl Default for Leave {
    fn default() -> Self {
        Leave {
            author: None,
            message_type: 12,
        }
    }
}

impl<'a> Parser<'a> for Leave {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Leave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n  Leave {{ 
             \n    message_type: {}
             \n  }}",
            self.message_type
        )
    }
}
