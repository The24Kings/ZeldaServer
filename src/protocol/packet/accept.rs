use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};

#[derive(Default, Debug, Clone)]
pub struct Accept {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub accept_type: u8,
}

impl<'a> Parser<'a> for Accept {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        // Implement serialization logic here
        Ok(())
    }

    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        // Implement deserialization logic here
        Ok(Self::default())
    }
}

impl std::fmt::Display for Accept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n  Accept {{ 
             \n    author: {:?},
             \n    message_type: {},
             \n    accept_type: {}
             \n  }}",
            self.author, self.message_type, self.accept_type
        )
    }
}
