use std::io::Write;

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
    Message(Message),
    ChangeRoom(ChangeRoom),
    Fight(Fight),
    PVPFight(PVPFight),
    Loot(Loot),
    Start(Start),
    Error(Error),
    Accept(Accept),
    Room(Room),
    Character(Character),
    Game(Game),
    Leave(Leave),
    Connection(Connection),
    Version(Version),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Message(msg) => write!(f, "\n{:#?}", msg),
            Type::ChangeRoom(room) => write!(f, "\n{:#?}", room),
            Type::Fight(fight) => write!(f, "\n{:#?}", fight),
            Type::PVPFight(pvp_fight) => write!(f, "\n{:#?}", pvp_fight),
            Type::Loot(loot) => write!(f, "\n{:#?}", loot),
            Type::Start(start) => write!(f, "\n{:#?}", start),
            Type::Error(error) => write!(f, "\n{:#?}", error),
            Type::Accept(accept) => write!(f, "\n{:#?}", accept),
            Type::Room(room) => write!(f, "\n{:#?}", room),
            Type::Character(character) => write!(f, "\n{:#?}", character),
            Type::Game(game) => write!(f, "\n{:#?}", game),
            Type::Leave(leave) => write!(f, "\n{:#?}", leave),
            Type::Connection(connection) => write!(f, "\n{:#?}", connection),
            Type::Version(version) => write!(f, "\n{:#?}", version),
        }
    }
}

pub fn send(packed: Type) -> Result<(), std::io::Error> {
    let mut byte_stream: Vec<u8> = Vec::new();

    println!("[SEND] Sending packet: {}", packed);

    // Serialize the packet and send it to the server
    let author = match packed {
        Type::Message(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::ChangeRoom(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Fight(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::PVPFight(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Loot(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Start(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Error(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Accept(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Room(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Character(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Game(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Leave(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Connection(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
        Type::Version(content) => {
            content.serialize(&mut byte_stream)?;
            content.author
        }
    };

    // Send the packet to the server
    match author {
        Some(author) => {
            author.as_ref().write_all(&byte_stream)?; // Send the packet to the server
        }
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No author found for message",
            ));
        }
    }

    Ok(())
}
