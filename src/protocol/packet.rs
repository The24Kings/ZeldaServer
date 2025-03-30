use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::{
    Packet, 
    Parser,
    SerializeError, 
    DeserializeError
};

use super::error::ErrorCode;

/** 
 * `author`: The client that sent the message
 * `message_type`: 1 byte - 0
 * `message_len`: 2 bytes - 1-2
 * `recipient`: 32 bytes - 3-34
 * `sender`: 32 bytes - 35-66
 * `message`: variable length - 67+
 
 Sent by the client to message other players. Can also be used by the server to send "presentable" information to the client 
 (information that can be displayed to the user with no further processing). Clients should expect to receive this type of message 
 at any time, and servers should expect to relay messages for clients at any time. If using this to send game information, 
 the server should mark the message as narration.
*/
#[derive(Debug, Clone)]
pub struct Message {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub message_len: u16,
    pub recipient: String,
    pub sender: String,
    pub message: String
}

impl Default for Message {
    fn default() -> Self {
        Message {
            author: None, // Replace with a valid default TcpStream if needed
            message_type: 1,
            message_len: 0,
            recipient: String::new(),
            sender: String::new(),
            message: String::new(),
        }
    }
}

impl<'a> Parser<'a> for Message {
    fn serialize<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        // Implement deserialization logic here
        Ok(Message {
            author: None,
            message_type: 1, 
            message_len: 0, 
            recipient: String::new(), 
            sender: String::new(), 
            message: String::new(), 
        })
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message {{ author: {:?}, message_type: {}, message_len: {:?}, recipient: {:?}, sender: {:?}, message: {:?} }}", self.author, self.message_type, self.message_len, self.recipient, self.sender, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct ChangeRoom {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub room_num: u16
}

impl Default for ChangeRoom {
    fn default() -> Self {
        ChangeRoom {
            author: None, 
            message_type: 2, 
            room_num: 0, 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fight {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
}

impl Default for Fight {
    fn default() -> Self {
        Fight {
            author: None, 
            message_type: 3, 
        }
    }
}

#[derive(Debug, Clone)]
pub struct PVPFight {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub target_name: String,
}

impl Default for PVPFight {
    fn default() -> Self {
        PVPFight {
            author: None, 
            message_type: 4, 
            target_name: String::new(), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Loot {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
    pub target_name: String,
}

impl Default for Loot {
    fn default() -> Self {
        Loot {
            author: None, 
            message_type: 5, 
            target_name: String::new(), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Start {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
}

impl Default for Start {
    fn default() -> Self {
        Start {
            author: None, 
            message_type: 6, 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub error: ErrorCode,
    pub message_len: u16,
    pub message: Vec<u8>
}

impl Default for Error {
    fn default() -> Self {
        Error {
            author: None, 
            message_type: 7, 
            error: ErrorCode::Other, 
            message_len: 30, 
            message: "Something went terribly wrong!".as_bytes().to_vec()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Accept {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub accept_type: u8,
}

impl Default for Accept {
    fn default() -> Self {
        Accept {
            author: None, 
            message_type: 8, 
            accept_type: 0, 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub room_number: Vec<u8>, // Same as room_num in ChangeRoom
    pub room_name: Vec<u8>,
    pub description_len: u16,
    pub description: Vec<u8>,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            author: None, 
            message_type: 9, 
            room_number: Vec::new(), 
            room_name: Vec::new(), 
            description_len: 0, 
            description: Vec::new(), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Character {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub name: String,
    pub flags: u8,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room: u16,
    pub description_len: u16,
    pub description: Vec<u8>,
}

impl Default for Character {
    fn default() -> Self {
        Character {
            author: None, 
            message_type: 10, 
            name: String::new(), 
            flags: 0x0, 
            attack: 0, 
            defense: 0, 
            regen: 0, 
            health: 0, 
            gold: 0, 
            current_room: 0, 
            description_len: 0, 
            description: Vec::new(), 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description_len: u16,
    pub description: String,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            author: None, 
            message_type: 11, 
            initial_points: 0,
            stat_limit: 0,
            description_len: 0,
            description: String::new(),
        }
    }
}

impl<'a> Parser<'a> for Game {
    fn serialize<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
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
            description: String::from_utf8(packet.body[6..(6 + description_len as usize)].to_vec()).unwrap_or_default()
        })
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game {{ initial_points: {}, stat_limit: {}, description_len: {}, description: {} }}", self.initial_points, self.stat_limit, self.description_len, self.description)
    }
}

#[derive(Debug, Clone)]
pub struct Leave {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
}

impl Default for Leave {
    fn default() -> Self {
        Leave {
            author: None,
            message_type: 12
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub room_number: u16,
    pub room_name: Vec<u8>,
    pub description_len: u16,
    pub description: Vec<u8>,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            author: None,
            message_type: 0,
            room_number: 0,
            room_name: Vec::new(),
            description_len: 0,
            description: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub author: Option<Arc<TcpStream>>,
    pub message_type: u8,
    pub major_rev: u8,
    pub minor_rev: u8,
    pub extension_len: u16, // Can be 0, just ignore
    pub extensions: Vec<u8>, // 0-1 length, 2-+ first extention;
}

impl Default for Version {
    fn default() -> Self {
        Version {
            author: None,
            message_type: 14,
            major_rev: 0,
            minor_rev: 0,
            extension_len: 0,
            extensions: Vec::new(), // 0-1 length, 2-+ first extention;
        }
    }
}

impl<'a> Parser<'a> for Version {
    fn serialize<W: Write>(&self, writer: W) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(packet: Packet) -> Result<Self, DeserializeError> {
        Ok(Version {
            author: packet.author,
            message_type: packet.message_type,
            major_rev: packet.body[0],
            minor_rev: packet.body[1],
            extension_len: 0,       //TODO: LurkCat doesn't send this, add it in the future
            extensions: Vec::new(), //TODO: LurkCat doesn't send this, add it in the future
        })
    }
}