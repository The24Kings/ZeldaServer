use lurk_lcsc::Protocol;
use std::sync::mpsc::Sender;

pub use crate::logic::commands::Action;
pub use crate::logic::config::Config;
pub use crate::logic::map::{Connection, Monster, Room};
pub use crate::logic::state::GameState;

pub mod commands;
pub mod config;
pub mod handlers;
pub mod map;
pub mod state;

pub enum ExtendedProtocol {
    Base(Protocol),
    Command(Action),
}

/// Type-safe wrapper around `Sender<ExtendedProtocol>`
pub struct GameSender(pub Sender<ExtendedProtocol>);

impl GameSender {
    pub fn send_base(&self, pkt: Protocol) {
        self.0
            .send(ExtendedProtocol::Base(pkt))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to send packet: {}", e);
            });
    }

    pub fn send_cmd(&self, action: Action) {
        let action_str = action.to_string();
        self.0
            .send(ExtendedProtocol::Command(action))
            .unwrap_or_else(|_| {
                tracing::error!("Failed to send {} packet", action_str);
            });
    }
}
