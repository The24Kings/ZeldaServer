use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{debug, info};

use crate::logic::{commands, pcap::PCap};
pub type Stream = Arc<TcpStream>;

pub use packet::{
    Packet, Parser, accept, change_room, character, connection, error, fight, game, leave, loot,
    message, pvp_fight, room, start, version,
};

pub mod flags;
pub mod lurk_error;
pub mod packet;
pub mod pkt_type;

pub enum Protocol {
    Message(Stream, message::PktMessage),
    ChangeRoom(Stream, change_room::PktChangeRoom),
    Fight(Stream, fight::PktFight),
    PVPFight(Stream, pvp_fight::PktPVPFight),
    Loot(Stream, loot::PktLoot),
    Start(Stream, start::PktStart),
    Error(Stream, error::PktError),
    Accept(Stream, accept::PktAccept),
    Room(Stream, room::PktRoom),
    Character(Stream, character::PktCharacter),
    Game(Stream, game::PktGame),
    Leave(Stream, leave::PktLeave),
    Connection(Stream, connection::PktConnection),
    Version(Stream, version::PktVersion),
    Command(commands::Action),
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Message(_, msg) => write!(f, "{}", msg),
            Protocol::ChangeRoom(_, room) => write!(f, "{}", room),
            Protocol::Fight(_, fight) => write!(f, "{}", fight),
            Protocol::PVPFight(_, pvp_fight) => write!(f, "{}", pvp_fight),
            Protocol::Loot(_, loot) => write!(f, "{}", loot),
            Protocol::Start(_, start) => write!(f, "{}", start),
            Protocol::Error(_, error) => write!(f, "{}", error),
            Protocol::Accept(_, accept) => write!(f, "{}", accept),
            Protocol::Room(_, room) => write!(f, "{}", room),
            Protocol::Character(_, character) => write!(f, "{}", character),
            Protocol::Game(_, game) => write!(f, "{}", game),
            Protocol::Leave(_, leave) => write!(f, "{}", leave),
            Protocol::Connection(_, connection) => write!(f, "{}", connection),
            Protocol::Version(_, version) => write!(f, "{}", version),
            Protocol::Command(action) => write!(f, "{}", action),
        }
    }
}

impl Protocol {
    pub fn send(self) -> Result<(), std::io::Error> {
        let mut byte_stream: Vec<u8> = Vec::new();

        info!("[PROTOCOL] Sending packet: {}", self);

        // Serialize the packet and send it to the server
        let author = match self {
            Protocol::Message(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::ChangeRoom(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Fight(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::PVPFight(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Loot(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Start(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Error(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Accept(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Room(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Character(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Game(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Leave(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Connection(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            Protocol::Version(author, content) => {
                content.serialize(&mut byte_stream)?;
                author
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot send this Protocol type",
                ));
            }
        };

        debug!("[PROTOCOL] Packet:\n{}", PCap::build(byte_stream.clone()));

        author.as_ref().write_all(&byte_stream)?;

        Ok(())
    }
}
