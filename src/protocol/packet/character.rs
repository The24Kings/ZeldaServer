use crate::{
    debug_packet,
    protocol::packet::{Packet, Parser},
};
use std::io::Write;
#[derive(Debug, Clone)]
pub struct Character {
    pub message_type: u8,
    pub name: String,
    pub flags: CharacterFlags,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description_len: u16,
    pub description: String,
}

impl Character {
    pub fn from(incoming: &Character) -> Self {
        Character {
            message_type: incoming.message_type,
            name: incoming.name.clone(),
            flags: CharacterFlags::default(),
            attack: incoming.attack,
            defense: incoming.defense,
            regen: incoming.regen,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: incoming.description_len,
            description: incoming.description.clone(),
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character {
            message_type: 10,
            name: "Error".to_string(),
            flags: CharacterFlags::default(),
            attack: 0,
            defense: 0,
            regen: 0,
            health: 100,
            gold: 0,
            current_room: 0,
            description_len: 60,
            description: "Something went wrong, please close the client and try again!".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharacterFlags {
    pub alive: bool,
    pub join_battle: bool,
    pub monster: bool,
    pub started: bool,
    pub ready: bool,
}

impl Default for CharacterFlags {
    fn default() -> Self {
        CharacterFlags {
            alive: true,
            join_battle: true,
            monster: false,
            started: false,
            ready: true,
        }
    }
}

impl<'a> Parser<'a> for Character {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);

        // Serialize the character name
        let mut name_bytes = self.name.as_bytes().to_vec();
        name_bytes.resize(32, 0x00); // Pad the name to 32 bytes

        packet.extend(name_bytes);

        // Serialize the flags byte in little-endian order
        let mut flags: u8 = 0x00;

        flags |= if self.flags.alive { 0b10000000 } else { 0x00 };
        flags |= if self.flags.join_battle { 0b01000000 } else { 0x00 };
        flags |= if self.flags.monster { 0b00100000 } else { 0x00 };
        flags |= if self.flags.started { 0b00010000 } else { 0x00 };
        flags |= if self.flags.ready { 0b00001000 } else { 0x00 };

        packet.extend(flags.to_le_bytes());

        // Serialize the character stats
        packet.extend(self.attack.to_le_bytes());
        packet.extend(self.defense.to_le_bytes());
        packet.extend(self.regen.to_le_bytes());
        packet.extend(self.health.to_le_bytes());
        packet.extend(self.gold.to_le_bytes());
        packet.extend(self.current_room.to_le_bytes());
        packet.extend(self.description_len.to_le_bytes());
        packet.extend(self.description.as_bytes());

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
        println!("[CHARACTER] Deserializing packet: {}", packet);

        let name = String::from_utf8_lossy(&packet.body[0..32])
            .trim_end_matches('\0')
            .to_string();
        let flags = packet.body[32];
        let attack = u16::from_le_bytes([packet.body[33], packet.body[34]]);
        let defense = u16::from_le_bytes([packet.body[35], packet.body[36]]);
        let regen = u16::from_le_bytes([packet.body[37], packet.body[38]]);
        let health = i16::from_le_bytes([packet.body[39], packet.body[40]]);
        let gold = u16::from_le_bytes([packet.body[41], packet.body[42]]);
        let current_room = u16::from_le_bytes([packet.body[43], packet.body[44]]);
        let description_len = u16::from_le_bytes([packet.body[45], packet.body[46]]);
        let description = String::from_utf8_lossy(&packet.body[47..]).to_string();

        // Parse the flags byte in little-endian order
        let flags = CharacterFlags {
            alive: flags & 0b10000000 != 0,
            join_battle: flags & 0b01000000 != 0,
            monster: flags & 0b00100000 != 0,
            started: flags & 0b00010000 != 0,
            ready: flags & 0b00001000 != 0,
        }; // Other bits are reserved for future use

        Ok(Character {
            message_type: packet.message_type,
            name,
            flags,
            attack,
            defense,
            regen,
            health,
            gold,
            current_room,
            description_len,
            description,
        })
    }
}
