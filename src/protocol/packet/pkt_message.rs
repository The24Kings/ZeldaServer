use std::io::Write;

use crate::{
    debug_packet,
    protocol::{
        packet::{Packet, Parser},
        pkt_type::PktType,
    },
};

#[derive(Default, Debug, Clone)]
pub struct Message {
    pub message_type: PktType,
    pub message_len: u16,
    pub recipient: String,
    pub sender: String,
    pub narration: bool,
    pub message: String,
}

impl<'a> Parser<'a> for Message {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type.into());
        packet.extend(self.message_len.to_le_bytes());

        let mut r_bytes = self.recipient.as_bytes().to_vec();
        let mut s_bytes = self.sender.as_bytes().to_vec();

        // Pad the recipient and sender names to 32 bytes
        r_bytes.resize(32, 0x00);
        s_bytes.resize(30, 0x00);

        // If the sender is a narrator, append 0x00 0x01 to the end of the sender name
        if self.narration {
            s_bytes.extend_from_slice(&[0x00, 0x01]);
        } else {
            s_bytes.resize(32, 0x00);
        }
        packet.extend(r_bytes);
        packet.extend(s_bytes);

        // Append the message
        packet.extend(self.message.as_bytes());

        // Write the packet to the buffer
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        debug_packet!(&packet);

        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        println!("[MESSAGE] Deserializing packet: {}", packet);

        let message_len = u16::from_le_bytes([packet.body[0], packet.body[1]]);

        // Process the names for recipient and sender
        let r_bytes = packet.body[2..34].to_vec();
        let mut s_bytes = packet.body[34..66].to_vec();

        // If the last 2 bytes of the sender is 0x00 0x01, it means the sender is a narrator
        let narration = match s_bytes.get(32..34) {
            Some(&[0x00, 0x01]) => {
                s_bytes.truncate(32); // Remove the last 2 bytes
                true
            }
            _ => false,
        };

        let recipient = String::from_utf8_lossy(&r_bytes)
            .split('\0')
            .collect::<Vec<&str>>()[0]
            .to_string();
        let sender = String::from_utf8_lossy(&s_bytes)
            .trim_end_matches('\0')
            .to_string();
        let message = String::from_utf8_lossy(&packet.body[66..]).to_string();

        Ok(Message {
            message_type: packet.message_type,
            message_len,
            recipient,
            sender,
            narration,
            message,
        })
    }
}
