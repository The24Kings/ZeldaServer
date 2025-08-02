use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::Parser;
use crate::protocol::packet::{
    pkt_accept, pkt_change_room, pkt_character, pkt_connection, pkt_error, pkt_fight, pkt_game,
    pkt_leave, pkt_loot, pkt_message, pkt_pvp_fight, pkt_room, pkt_start, pkt_version,
};

pub type Stream = Arc<TcpStream>;

pub mod client;
pub mod error;
pub mod map;
pub mod packet;
pub mod pkt_type;

pub enum ServerMessage {
    Message(Stream, pkt_message::Message),
    ChangeRoom(Stream, pkt_change_room::ChangeRoom),
    Fight(Stream, pkt_fight::Fight),
    PVPFight(Stream, pkt_pvp_fight::PVPFight),
    Loot(Stream, pkt_loot::Loot),
    Start(Stream, pkt_start::Start),
    Error(Stream, pkt_error::Error),
    Accept(Stream, pkt_accept::Accept),
    Room(Stream, pkt_room::Room),
    Character(Stream, pkt_character::Character),
    Game(Stream, pkt_game::Game),
    Leave(Stream, pkt_leave::Leave),
    Connection(Stream, pkt_connection::Connection),
    Version(Stream, pkt_version::Version),
}

impl std::fmt::Display for ServerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMessage::Message(_, msg) => write!(f, "\n{:#?}", msg),
            ServerMessage::ChangeRoom(_, room) => write!(f, "\n{:#?}", room),
            ServerMessage::Fight(_, fight) => write!(f, "\n{:#?}", fight),
            ServerMessage::PVPFight(_, pvp_fight) => write!(f, "\n{:#?}", pvp_fight),
            ServerMessage::Loot(_, loot) => write!(f, "\n{:#?}", loot),
            ServerMessage::Start(_, start) => write!(f, "\n{:#?}", start),
            ServerMessage::Error(_, error) => write!(f, "\n{:#?}", error),
            ServerMessage::Accept(_, accept) => write!(f, "\n{:#?}", accept),
            ServerMessage::Room(_, room) => write!(f, "\n{:#?}", room),
            ServerMessage::Character(_, character) => write!(f, "\n{:#?}", character),
            ServerMessage::Game(_, game) => write!(f, "\n{:#?}", game),
            ServerMessage::Leave(_, leave) => write!(f, "\n{:#?}", leave),
            ServerMessage::Connection(_, connection) => write!(f, "\n{:#?}", connection),
            ServerMessage::Version(_, version) => write!(f, "\n{:#?}", version),
        }
    }
}

pub fn send(packed: ServerMessage) -> Result<(), std::io::Error> {
    let mut byte_stream: Vec<u8> = Vec::new();

    println!("[SEND] Sending packet: {}", packed);

    // Serialize the packet and send it to the server
    let author = match packed {
        ServerMessage::Message(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::ChangeRoom(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Fight(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::PVPFight(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Loot(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Start(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Error(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Accept(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Room(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Character(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Game(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Leave(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Connection(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        ServerMessage::Version(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
    };

    author.as_ref().write_all(&byte_stream).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            format!("Failed to write to stream: {}", e),
        )
    })?;

    Ok(())
}
