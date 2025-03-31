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

pub fn send(packed: Type) -> Result<(), std::io::Error> {
    let author: Option<Arc<TcpStream>>;
    let mut byte_stream: Vec<u8> = Vec::new();

    println!("[SEND] Sending packet: {:?}", packed);

    // Serialize the packet and send it to the server
    match packed {
        Type::Message( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?;
        },
        Type::ChangeRoom( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Fight( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::PVPFight( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Loot( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Start( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Error( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Accept( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Room( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Character( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Game( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Leave( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Connection( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
        },
        Type::Version( content ) => {
            author = content.author.clone();
            content.serialize(&mut byte_stream)?
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