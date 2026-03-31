use lurk_lcsc::PktError;
use lurk_lcsc::PktPVPFight;
use lurk_lcsc::{LurkError, Protocol};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::state::GameState;

impl GameState {
    pub fn handle_pvp_fight(&self, author: Arc<TcpStream>, content: PktPVPFight) {
        info!("Received: {}", content);

        Protocol::Error(
            author.clone(),
            PktError::new(LurkError::NOPLAYERCOMBAT, "No player combat allowed"),
        )
        .send()
        .unwrap_or_else(|e| {
            error!("Failed to send error packet: {}", e);
        });
    }
}
