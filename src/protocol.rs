use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

pub mod packet;
pub mod error;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Type {
    Message(packet::Message) = 1,
    ChangeRoom(packet::ChangeRoom) = 2,
    Fight(packet::Fight) = 3,
    PVPFight(packet::PVPFight) = 4,
    Loot(packet::Loot) = 5,
    Start(packet::Start) = 6,
    Error(packet::Error) = 7,
    Accept(packet::Accept) = 8,
    Room(packet::Room) = 9,
    Character(packet::Character) = 10,
    Game(packet::Game) = 11,
    Leave(packet::Leave) = 12,
    Connection(packet::Connection) = 13,
    Version(packet::Version) = 14,
}

/**
 * Packet structure used for passing data between the server and client at a low level
 * message_type: Type of the packet
 * body: Body of the packet
 */
#[derive(Debug)]
pub struct Packet<'a> {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub body: &'a[u8]
}

impl<'a> Packet<'a> {
    pub fn new(author: Arc<TcpStream>, id: u8, bytes: &'a[u8]) -> Self {
        Packet {
            author: Some(author),
            message_type: id,
            body: &bytes[0..]
        }
    }

    pub fn read_into<'b>(stream: Arc<TcpStream>, id: u8, buffer: &'b mut Vec<u8>) -> Result<Packet<'b>, std::io::Error> {
        // Read the remaining bytes for the packet
        let _bytes_read = stream.as_ref().read( buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Create a new packet with the read bytes
        let packet = Packet::new(stream, id, buffer);

        Ok(packet)
    }

    pub fn read_with_desc<'b>(stream: Arc<TcpStream>, id: u8, buffer: &'b mut Vec<u8>, index: (u8, u8)) -> Result<Packet<'b>, std::io::Error> {
        let _partial_bytes_read = stream.as_ref().read( buffer).map_err(|e| {
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
    fn serialize<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
        Ok(())
    }
    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        Ok(Self::default())
    }
}

#[derive(Debug, Clone)]
pub struct SerializeError {
    message: String,
}

#[derive(Debug, Clone)]
pub struct DeserializeError {
    message: String,
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error serializing message: {}", self.message)
    }
}

impl SerializeError {
    pub fn new() -> Self {
        SerializeError {
            message: String::from("Serialization error"),
        }
    }

    pub fn with_message(message: &str) -> Self {
        SerializeError {
            message: String::from(message),
        }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error deserializing message: {}", self.message)
    }
}

impl DeserializeError {
    pub fn new() -> Self {
        DeserializeError {
            message: String::from("Deserialization error"),
        }
    }

    pub fn with_message(message: &str) -> Self {
        DeserializeError {
            message: String::from(message),
        }
    }
}
