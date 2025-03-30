use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::{Packet, Parser};
use crate::protocol::parsing_error::{DeserializeError, SerializeError};

#[derive(Default, Debug, Clone)]
pub struct Game {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description_len: u16,
    pub description: String,
}

impl<'a> Parser<'a> for Game {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        let initial_points = u16::from_le_bytes([packet.body[0], packet.body[1]]);
        let stat_limit = u16::from_le_bytes([packet.body[2], packet.body[3]]);
        let description_len = u16::from_le_bytes([packet.body[4], packet.body[5]]);

        Ok(Game {
            author: packet.author,
            message_type: packet.message_type,
            initial_points,
            stat_limit,
            description_len,
            description: String::from_utf8(packet.body[6..(6 + description_len as usize)].to_vec())
                .unwrap_or_default(),
        })
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Game {{ initial_points: {}, stat_limit: {}, description_len: {}, description: {} }}",
            self.initial_points, self.stat_limit, self.description_len, self.description
        )
    }
}
