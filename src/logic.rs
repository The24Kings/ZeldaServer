use lurk_lcsc::Protocol;
use lurk_sansio::ClientId;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

pub use crate::logic::command::{Command, handle_command};
pub use crate::logic::config::Config;
pub use crate::logic::execute::execute_output;
pub use crate::logic::translate::translate;

pub mod command;
pub mod config;
pub mod execute;
pub mod translate;

pub enum ExtendedProtocol {
    Connect(ClientId, Arc<TcpStream>),
    Client(ClientId, Protocol),
    Command(Command),
}

/// Type-safe wrapper around `Sender<ExtendedProtocol>`
pub struct GameSender(pub Sender<ExtendedProtocol>);

impl GameSender {
    pub fn send_connect(&self, id: ClientId, stream: Arc<TcpStream>) {
        self.0
            .send(ExtendedProtocol::Connect(id, stream))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to send connect: {}", e);
            });
    }

    pub fn send_client(&self, id: ClientId, pkt: Protocol) {
        self.0
            .send(ExtendedProtocol::Client(id, pkt))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to send packet: {}", e);
            });
    }

    pub fn send_cmd(&self, action: Command) {
        let action_str = action.to_string();
        self.0
            .send(ExtendedProtocol::Command(action))
            .unwrap_or_else(|_| {
                tracing::error!("Failed to send {} packet", action_str);
            });
    }
}
