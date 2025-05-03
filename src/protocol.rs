use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use crate::protocol::packet::Parser;

use crate::protocol::packet::{
    accept::Accept, change_room::ChangeRoom, character::Character, connection::Connection,
    error::Error, fight::Fight, game::Game, leave::Leave, loot::Loot, message::Message,
    pvp_fight::PVPFight, room::Room, start::Start, version::Version,
};

pub mod client;
pub mod error;
pub mod packet;
pub mod map;

#[derive(Debug, Clone)]
pub enum Type {
    Message(Arc<TcpStream>, Message),
    ChangeRoom(Arc<TcpStream>, ChangeRoom),
    Fight(Arc<TcpStream>, Fight),
    PVPFight(Arc<TcpStream>, PVPFight),
    Loot(Arc<TcpStream>, Loot),
    Start(Arc<TcpStream>, Start),
    Error(Arc<TcpStream>, Error),
    Accept(Arc<TcpStream>, Accept),
    Room(Arc<TcpStream>, Room),
    Character(Arc<TcpStream>, Character),
    Game(Arc<TcpStream>, Game),
    Leave(Arc<TcpStream>, Leave),
    Connection(Arc<TcpStream>, Connection),
    Version(Arc<TcpStream>, Version),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Message(_, msg) => write!(f, "\n{:#?}", msg),
            Type::ChangeRoom(_, room) => write!(f, "\n{:#?}", room),
            Type::Fight(_, fight) => write!(f, "\n{:#?}", fight),
            Type::PVPFight(_, pvp_fight) => write!(f, "\n{:#?}", pvp_fight),
            Type::Loot(_, loot) => write!(f, "\n{:#?}", loot),
            Type::Start(_, start) => write!(f, "\n{:#?}", start),
            Type::Error(_, error) => write!(f, "\n{:#?}", error),
            Type::Accept(_, accept) => write!(f, "\n{:#?}", accept),
            Type::Room(_, room) => write!(f, "\n{:#?}", room),
            Type::Character(_, character) => write!(f, "\n{:#?}", character),
            Type::Game(_, game) => write!(f, "\n{:#?}", game),
            Type::Leave(_, leave) => write!(f, "\n{:#?}", leave),
            Type::Connection(_, connection) => write!(f, "\n{:#?}", connection),
            Type::Version(_, version) => write!(f, "\n{:#?}", version),
        }
    }
}

pub fn send(packed: Type) -> Result<(), std::io::Error> {
    let mut byte_stream: Vec<u8> = Vec::new();

    println!("[SEND] Sending packet: {}", packed);

    // Serialize the packet and send it to the server
    let author = match packed {
        Type::Message(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::ChangeRoom(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Fight(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::PVPFight(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Loot(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Start(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Error(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Accept(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Room(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Character(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Game(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Leave(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Connection(author, content) => {
            content.serialize(&mut byte_stream)?;
            author
        }
        Type::Version(author, content) => {
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
