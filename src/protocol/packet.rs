use std::io::{Read, Write};
use tracing::debug;

use crate::protocol::{Stream, pcap::PCap, pkt_type::PktType};

pub mod pkt_accept;
pub mod pkt_change_room;
pub mod pkt_character;
pub mod pkt_connection;
pub mod pkt_error;
pub mod pkt_fight;
pub mod pkt_game;
pub mod pkt_leave;
pub mod pkt_loot;
pub mod pkt_message;
pub mod pkt_pvp_fight;
pub mod pkt_room;
pub mod pkt_start;
pub mod pkt_version;

pub trait Parser<'a>: Sized + 'a {
    fn serialize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error>;
    fn deserialize(packet: Packet) -> Result<Self, std::io::Error>;
}

pub struct Packet<'a> {
    pub stream: &'a Stream,
    pub message_type: PktType,
    pub body: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(stream: &'a Stream, message_type: PktType, bytes: &'a [u8]) -> Self {
        Packet {
            stream,
            message_type,
            body: &bytes[0..],
        }
    }

    /// Read the stream into a packet
    pub fn read_into<'b>(
        stream: &'b Stream,
        message_type: PktType,
        buffer: &'b mut Vec<u8>,
    ) -> Result<Packet<'b>, std::io::Error> {
        // Read the remaining bytes for the packet
        let _bytes_read = stream.as_ref().read_exact(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Print the packet body
        debug!("[DEBUG] Packet body:\n{}", PCap::build(buffer.clone()));

        // Create a new packet with the read bytes
        let packet = Packet::new(stream, message_type, buffer);

        Ok(packet)
    }

    /// Read the packet with a varied length.
    /// This function reads the packet body and then reads the extended description or data
    /// based on the provided index.
    pub fn read_extended<'b>(
        stream: &'b Stream,
        message_type: PktType,
        buffer: &'b mut Vec<u8>,
        index: (usize, usize),
    ) -> Result<Packet<'b>, std::io::Error> {
        stream.as_ref().read_exact(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        // Get the description length from the buffer
        let length = usize::from_le_bytes([buffer[index.0], buffer[index.1], 0, 0, 0, 0, 0, 0]);
        let mut desc = vec![0u8; length];

        debug!(
            "[PACKET] Reading description of length {} at index {}, {}",
            length, index.0, index.1
        );

        // Read the description from the stream
        stream.as_ref().read_exact(&mut desc).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read descriptor: {}", e),
            )
        })?;

        // Print the description
        if !desc.is_empty() {
            debug!(
                "[PACKET] Read description: {}",
                String::from_utf8_lossy(&desc)
            );
        } else {
            debug!("[PACKET] Read empty description");
        }

        // Extend the buffer with the description
        buffer.extend_from_slice(&desc);

        // Print the packet body
        debug!("[DEBUG] Packet body:\n{}", PCap::build(buffer.clone()));

        let packet = Packet::new(stream, message_type, buffer);

        Ok(packet)
    }
}
