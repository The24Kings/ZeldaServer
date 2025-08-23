use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{debug, info};

use crate::commands;
use crate::protocol::packet::*;
use crate::protocol::pcap::PCap;

pub type Stream = Arc<TcpStream>;

pub mod client;
pub mod error;
pub mod game;
pub mod packet;
pub mod pcap;
pub mod pkt_type;

pub enum Protocol {
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
    pub fn send(&self) -> Result<(), std::io::Error> {
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
