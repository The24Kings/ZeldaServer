use std::io::{Read, Write};

use crate::protocol::Stream;

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

/**
 * Packet structure used for passing data between the server and client at a low level
 * message_type: Type of the packet
 * body: Body of the packet
 */
#[derive(Debug, Clone)]
pub struct Packet<'a> {
    pub stream: &'a Stream,
    pub message_type: u8,
    pub body: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(stream: &'a Stream, id: u8, bytes: &'a [u8]) -> Self {
        Packet {
            stream,
            message_type: id,
            body: &bytes[0..],
        }
    }

    /// Read the stream into a packet
    pub fn read_into<'b>(
        stream: &'b Stream,
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

        println!(
            "[PACKET] Read packet body: {}",
            buffer
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );
        // Create a new packet with the read bytes
        let packet = Packet::new(stream, id, buffer);

        Ok(packet)
    }

    /// Read the packet with a varied length.
    /// This function reads the packet body and then reads the extended description or data
    /// based on the provided index.
    pub fn read_extended<'b>(
        stream: &'b Stream,
        id: u8,
        buffer: &'b mut Vec<u8>,
        index: (usize, usize),
    ) -> Result<Packet<'b>, std::io::Error> {
        stream.as_ref().read(buffer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Failed to read packet body: {}", e),
            )
        })?;

        println!(
            "[PACKET] Read packet body: {}",
            buffer
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        // Get the description length from the buffer
        let length = usize::from_le_bytes([buffer[index.0], buffer[index.1], 0, 0, 0, 0, 0, 0]);
        let mut desc = vec![0u8; length];

        println!(
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
        let desc_str = String::from_utf8_lossy(&desc);

        println!(
            "[PACKET] Read description: {}",
            String::from(if desc.is_empty() {
                "No description provided"
            } else {
                &desc_str
            })
        );

        // Extend the buffer with the description
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
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<(), std::io::Error> {
        Ok(())
    }
    fn deserialize(_packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Self::default())
    }
}

impl<'a> std::fmt::Display for Packet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nPacket {{\n    message_type: {},\n    body: [\n        {}\n    ]\n}}",
            self.message_type,
            self.body
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

/// Debug function to print the packet in a human readable format
/// This function prints the first 64 bytes of the packet
#[macro_export]
macro_rules! debug_packet {
    ($packet:expr) => {{
        println!(
            "[DEBUG] Serialized packet: {}",
            $packet
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        ); // TODO: Add another field for the message; (message, packet)
    }};
}
