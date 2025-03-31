use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::parsing_error::{DeserializeError, SerializeError};

pub mod accept;
pub mod change_room;
pub mod character;
pub mod connection;
pub mod error;
pub mod fight;
pub mod game;
pub mod leave;
pub mod loot;
pub mod message;
pub mod pvp_fight;
pub mod room;
pub mod start;
pub mod version;

/**
 * Packet structure used for passing data between the server and client at a low level
 * message_type: Type of the packet
 * body: Body of the packet
 */
#[derive(Debug, Clone)]
pub struct Packet<'a> {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub body: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(author: Arc<TcpStream>, id: u8, bytes: &'a [u8]) -> Self {
        Packet {
            author: Some(author),
            message_type: id,
            body: &bytes[0..],
        }
    }

    /// Read the stream into a packet
    pub fn read_into<'b>(
        stream: Arc<TcpStream>,
        id: u8,
        buffer: &'b mut Vec<u8>,
    ) -> Result<Packet<'b>, std::io::Error> {
        // Read the remaining bytes for the packet
        let _bytes_read = stream.as_ref().read(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Create a new packet with the read bytes
        let packet = Packet::new(stream, id, buffer);

        Ok(packet)
    }

    /// Read the packet with a description.
    /// This function reads the packet body and then reads the description
    pub fn read_with_desc<'b>(
        stream: Arc<TcpStream>,
        id: u8,
        buffer: &'b mut Vec<u8>,
        index: (u8, u8),
    ) -> Result<Packet<'b>, std::io::Error> {
        stream.as_ref().read(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Get the description
        let length = u16::from_le_bytes([index.0, index.1]);
        let mut desc = vec![0u8; length as usize];

        stream.as_ref().read_exact(&mut desc).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read descriptor: {}", e),
            )
        })?;

        // Combine into the packet
        buffer.extend_from_slice(&desc);

        let packet = Packet::new(stream, id, buffer);

        Ok(packet)
    }
}

/**
 * Trait imlementation for the packet
 * Serialize: Serialize the packet to a byte array
 * Deserialize: Deserialize the packet from a byte array
 * Display: Display the packet in a human readable format
 */
pub trait Parser<'a>: Sized + 'a + Default {
    fn serialize<W: Write>(&self, _writer: W) -> Result<(), SerializeError> {
        Ok(())
    }
    fn deserialize(_packet: Packet) -> Result<Self, DeserializeError> {
        Ok(Self::default())
    }
}
